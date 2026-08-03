//! HTTP client for MARA's Slipstream direct-submission API.
//!
//! This is the only crate that ever holds the Slipstream client code: it is
//! read out of config at construction, used to build the `/api/transactions`
//! request body, and never handed to a caller or written to a log line. See
//! [`SlipstreamError`] for why the client code must never appear in error
//! output either.

use core::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};

const MAX_RESPONSE_BODY_BYTES: usize = 64 * 1024;

/// Configuration needed to construct a [`SlipstreamClient`], taken verbatim
/// from `/etc/outofband/config.toml`'s `[slipstream]` table. Endpoint paths
/// are config, not code, so a change on MARA's side (the API is
/// self-described as beta) is an edit plus a restart rather than a rebuild.
#[derive(Clone)]
pub struct SlipstreamConfig {
    pub base_url: String,
    pub fee_endpoint: String,
    pub submit_endpoint: String,
    /// Empty means "not configured": `rates()` omits the discount query
    /// parameter and `submit_tx()` fails locally with
    /// [`SlipstreamError::ClientCode`] rather than making a request MARA
    /// would reject anyway.
    pub client_code: String,
    pub request_timeout_secs: u64,
}

impl fmt::Debug for SlipstreamConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlipstreamConfig")
            .field("base_url", &self.base_url)
            .field("fee_endpoint", &self.fee_endpoint)
            .field("submit_endpoint", &self.submit_endpoint)
            .field(
                "client_code",
                &if self.client_code.is_empty() {
                    "<none>"
                } else {
                    "<redacted>"
                },
            )
            .field("request_timeout_secs", &self.request_timeout_secs)
            .finish()
    }
}

/// Client for the two Slipstream endpoints this project uses:
/// `GET /api/rates` and `POST /api/transactions`. The sole holder of the
/// client code — see the module docs.
pub struct SlipstreamClient {
    http: Client,
    base_url: String,
    fee_endpoint: String,
    submit_endpoint: String,
    client_code: Option<String>,
}

impl fmt::Debug for SlipstreamClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlipstreamClient")
            .field("base_url", &self.base_url)
            .field("fee_endpoint", &self.fee_endpoint)
            .field("submit_endpoint", &self.submit_endpoint)
            .field(
                "client_code",
                &self.client_code.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl SlipstreamClient {
    /// Builds the client from config. Panics only if the underlying HTTP
    /// client cannot be built at all (an unrecoverable startup failure, not
    /// a per-request condition).
    pub fn new(config: &SlipstreamConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url: config.base_url.clone(),
            fee_endpoint: config.fee_endpoint.clone(),
            submit_endpoint: config.submit_endpoint.clone(),
            client_code: if config.client_code.is_empty() {
                None
            } else {
                Some(config.client_code.clone())
            },
        }
    }

    /// `GET {base_url}{fee_endpoint}`, appending `?client_code=…` only when
    /// one is configured (it applies volume discounts). Needs no
    /// credential and works without one.
    pub async fn rates(&self) -> Result<FeeInfo, SlipstreamError> {
        let url = format!("{}{}", self.base_url, self.fee_endpoint);
        let mut request = self.http.get(&url);
        if let Some(code) = &self.client_code {
            request = request.query(&[("client_code", code)]);
        }

        let response = request
            .send()
            .await
            .map_err(|err| self.transport_error(err))?;
        let (status, body) = self.read_response(response).await?;

        if status.is_success() {
            let parsed: RatesResponse =
                serde_json::from_str(&body).map_err(|_| SlipstreamError::Http {
                    status: status.as_u16(),
                    body: "could not parse rates response".to_string(),
                })?;
            return Ok(FeeInfo {
                effective_rate_sat_vb: parsed.effective_rate,
                fetched_at: Utc::now(),
            });
        }

        match serde_json::from_str::<RatesErrorBody>(&body) {
            Ok(err) => Err(SlipstreamError::Rejected(self.redact(&err.message))),
            Err(_) => Err(SlipstreamError::Http {
                status: status.as_u16(),
                body: "could not parse rates error response".to_string(),
            }),
        }
    }

    /// `POST {base_url}{submit_endpoint}` with `{client_code, tx_hex}`, the
    /// body built entirely inside this crate — no other module ever sees
    /// the code. Never sets `skip_mempool_submission`.
    pub async fn submit_tx(&self, tx_hex: &str) -> Result<SubmitResult, SlipstreamError> {
        let Some(client_code) = &self.client_code else {
            return Err(SlipstreamError::ClientCode(
                "no client code configured".to_string(),
            ));
        };

        let url = format!("{}{}", self.base_url, self.submit_endpoint);
        let body = SubmitRequestBody {
            client_code,
            tx_hex,
        };

        let response = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|err| self.transport_error(err))?;
        let (status, text) = self.read_response(response).await?;

        let parsed: SubmitResponseBody =
            serde_json::from_str(&text).map_err(|_| SlipstreamError::Http {
                status: status.as_u16(),
                body: "could not parse submission response".to_string(),
            })?;

        if status.is_success() && parsed.status == "success" {
            return Ok(SubmitResult {
                status: parsed.status,
                message: self.redact(&parsed.message),
            });
        }

        let client_code_issue = is_client_code_issue(&parsed.message);
        let message = self.redact(&parsed.message);
        if client_code_issue {
            return Err(SlipstreamError::ClientCode(message));
        }
        if !status.is_success() {
            return Err(SlipstreamError::Http {
                status: status.as_u16(),
                body: message,
            });
        }
        if parsed.status == "error" {
            return Err(SlipstreamError::Rejected(message));
        }

        Err(SlipstreamError::Http {
            status: status.as_u16(),
            body: message,
        })
    }

    async fn read_response(
        &self,
        mut response: Response,
    ) -> Result<(StatusCode, String), SlipstreamError> {
        let status = response.status();
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|err| self.transport_error(err))?
        {
            if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BODY_BYTES {
                return Err(SlipstreamError::Http {
                    status: status.as_u16(),
                    body: format!("response body exceeds {MAX_RESPONSE_BODY_BYTES} bytes"),
                });
            }
            body.extend_from_slice(&chunk);
        }

        Ok((status, String::from_utf8_lossy(&body).into_owned()))
    }

    fn transport_error(&self, err: reqwest::Error) -> SlipstreamError {
        SlipstreamError::Transport(self.redact(&err.without_url().to_string()))
    }

    fn redact(&self, message: &str) -> String {
        match &self.client_code {
            Some(client_code) => message.replace(client_code, "<redacted>"),
            None => message.to_string(),
        }
    }
}

/// The only substring MARA's documented "client codes are currently
/// required" message is known to contain, matched case-insensitively. It is
/// the one submission-rejection signal this client can attribute reliably
/// to the client code rather than to the transaction itself; anything else
/// falls through to [`SlipstreamError::Rejected`].
fn is_client_code_issue(message: &str) -> bool {
    message.to_ascii_lowercase().contains("client code")
}

/// The fee numbers from `GET /api/rates`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeeInfo {
    /// `effective_rate`: the rate at which a submission actually gets
    /// mined, as opposed to `submit_fee_rate`, which only buys admission
    /// to Slipstream's private mempool.
    pub effective_rate_sat_vb: f64,
    pub fetched_at: DateTime<Utc>,
}

/// Slipstream's answer to a submission, carried verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitResult {
    pub status: String,
    pub message: String,
}

/// Everything that can go wrong talking to Slipstream.
#[derive(Debug)]
pub enum SlipstreamError {
    /// The request could not be sent or the response could not be read —
    /// DNS, TCP, TLS, timeout. Carries a message with the request URL
    /// stripped (see [`transport_error`]), never a raw [`reqwest::Error`],
    /// so a failed `rates()` call can never leak the client code through
    /// its query parameter.
    Transport(String),
    /// The response did not match either known JSON error shape for its
    /// HTTP status, or the status itself was unexpected.
    Http { status: u16, body: String },
    /// No client code is configured, or Slipstream's rejection message
    /// names the client code as the problem (its documented "client codes
    /// are currently required" wording is the only submission-rejection
    /// text this client can attribute to the code rather than to the
    /// transaction).
    ClientCode(String),
    /// Slipstream rejected the submission for a reason that is not the
    /// client code; `message` is its prose, verbatim.
    ///
    /// Fee-too-low and consensus-invalid rejections are **not**
    /// distinguishable here: `/api/transactions` reports both as free-text
    /// `message` on the same `{status:"error", ...}` shape, so this
    /// variant covers both and callers must show `message` to the user
    /// rather than branch on it.
    Rejected(String),
}

impl fmt::Display for SlipstreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlipstreamError::Transport(msg) => write!(f, "transport error: {msg}"),
            SlipstreamError::Http { status, body } => {
                write!(f, "unexpected response (HTTP {status}): {body}")
            }
            SlipstreamError::ClientCode(msg) => write!(f, "client code error: {msg}"),
            SlipstreamError::Rejected(msg) => write!(f, "rejected: {msg}"),
        }
    }
}

impl std::error::Error for SlipstreamError {}

#[derive(Deserialize)]
struct RatesResponse {
    effective_rate: f64,
}

/// Error shape returned by `/api/rates` (and `/api/transactions/status`):
/// distinct from [`SubmitResponseBody`], which submission uses.
#[derive(Deserialize)]
struct RatesErrorBody {
    message: String,
}

/// The `{status, message}` shape returned by `/api/transactions` on both
/// 200 and 400.
#[derive(Deserialize)]
struct SubmitResponseBody {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct SubmitRequestBody<'a> {
    client_code: &'a str,
    tx_hex: &'a str,
}
