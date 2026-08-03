//! Configuration for `broadcast-api`, loaded from a TOML file whose path is
//! the first CLI argument, falling back to [`DEFAULT_CONFIG_PATH`] — this is
//! how the systemd unit invokes the binary. Missing optional keys fall back
//! to the defaults documented in `deploy/config.toml`; a missing or
//! malformed file is a startup failure, never a partially-applied config.

use std::fmt;
use std::net::{AddrParseError, SocketAddr};
use std::time::Duration;

use serde::Deserialize;
use slipstream_client::SlipstreamConfig;

pub const DEFAULT_CONFIG_PATH: &str = "/etc/outofband/config.toml";

/// `Debug` is safe to derive here: `SlipstreamConfig`'s own `Debug` impl
/// already redacts the client code, and `BroadcastApiConfig` holds no
/// secrets. Application code must still never log a whole `Config`.
#[derive(Debug)]
pub struct Config {
    pub slipstream: SlipstreamConfig,
    pub broadcast_api: BroadcastApiConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BroadcastApiConfig {
    pub listen_addr: SocketAddr,
    pub fee_poll_secs: Duration,
    pub rate_limit_window_secs: Duration,
    pub rate_limit_max_tx: usize,
    pub max_payload_bytes: usize,
}

/// Everything that can go wrong loading the config. `Display` never embeds
/// the file's contents, only the path and the underlying parse/IO error, so
/// a startup failure can never echo the client code back in a log line.
#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
    },
    InvalidListenAddr {
        value: String,
        source: AddrParseError,
    },
    InvalidValue {
        key: &'static str,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Read { path, source } => {
                write!(f, "could not read config file {path}: {source}")
            }
            ConfigError::Parse { path } => write!(f, "could not parse config file {path}"),
            ConfigError::InvalidListenAddr { value, source } => {
                write!(f, "invalid broadcast_api.listen_addr {value:?}: {source}")
            }
            ConfigError::InvalidValue { key } => write!(f, "{key} must be greater than zero"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// The config path: the first CLI argument, or [`DEFAULT_CONFIG_PATH`].
pub fn config_path() -> String {
    std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string())
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_string(),
        source,
    })?;
    parse_config(&text, path)
}

fn parse_config(text: &str, path: &str) -> Result<Config, ConfigError> {
    let raw: RawConfig = toml::from_str(text).map_err(|source| ConfigError::Parse {
        path: path.to_string(),
        source,
    })?;

    let listen_addr =
        raw.broadcast_api
            .listen_addr
            .parse()
            .map_err(|source| ConfigError::InvalidListenAddr {
                value: raw.broadcast_api.listen_addr.clone(),
                source,
            })?;

    Ok(Config {
        slipstream: SlipstreamConfig {
            base_url: raw.slipstream.base_url,
            fee_endpoint: raw.slipstream.fee_endpoint,
            submit_endpoint: raw.slipstream.submit_endpoint,
            client_code: raw.slipstream.client_code,
            request_timeout_secs: raw.slipstream.request_timeout_secs,
        },
        broadcast_api: BroadcastApiConfig {
            listen_addr,
            fee_poll_secs: Duration::from_secs(raw.broadcast_api.fee_poll_secs),
            rate_limit_window_secs: Duration::from_secs(raw.broadcast_api.rate_limit_window_secs),
            rate_limit_max_tx: raw.broadcast_api.rate_limit_max_tx,
            max_payload_bytes: raw.broadcast_api.max_payload_bytes,
        },
    })
}

#[derive(Deserialize)]
struct RawConfig {
    slipstream: RawSlipstreamConfig,
    #[serde(default)]
    broadcast_api: RawBroadcastApiConfig,
}

#[derive(Deserialize)]
struct RawSlipstreamConfig {
    base_url: String,
    fee_endpoint: String,
    submit_endpoint: String,
    #[serde(default)]
    client_code: String,
    #[serde(default = "default_request_timeout_secs")]
    request_timeout_secs: u64,
}

fn default_request_timeout_secs() -> u64 {
    30
}

#[derive(Deserialize)]
#[serde(default)]
struct RawBroadcastApiConfig {
    listen_addr: String,
    fee_poll_secs: u64,
    rate_limit_window_secs: u64,
    rate_limit_max_tx: usize,
    max_payload_bytes: usize,
}

impl Default for RawBroadcastApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:3010".to_string(),
            fee_poll_secs: 60,
            rate_limit_window_secs: 600,
            rate_limit_max_tx: 100,
            max_payload_bytes: 1_048_576,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_config() {
        let text = r#"
[slipstream]
base_url = "https://slipstream.mara.com"
fee_endpoint = "/api/rates"
submit_endpoint = "/api/transactions"
client_code = "secret"
request_timeout_secs = 15

[broadcast_api]
listen_addr = "127.0.0.1:4000"
fee_poll_secs = 30
rate_limit_window_secs = 300
rate_limit_max_tx = 50
max_payload_bytes = 2097152
"#;
        let config = parse_config(text, "test.toml").unwrap();
        assert_eq!(config.slipstream.base_url, "https://slipstream.mara.com");
        assert_eq!(config.slipstream.fee_endpoint, "/api/rates");
        assert_eq!(config.slipstream.submit_endpoint, "/api/transactions");
        assert_eq!(config.slipstream.client_code, "secret");
        assert_eq!(config.slipstream.request_timeout_secs, 15);
        assert_eq!(
            config.broadcast_api.listen_addr,
            "127.0.0.1:4000".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.broadcast_api.fee_poll_secs, Duration::from_secs(30));
        assert_eq!(
            config.broadcast_api.rate_limit_window_secs,
            Duration::from_secs(300)
        );
        assert_eq!(config.broadcast_api.rate_limit_max_tx, 50);
        assert_eq!(config.broadcast_api.max_payload_bytes, 2_097_152);
    }

    #[test]
    fn missing_optional_keys_use_defaults() {
        let text = r#"
[slipstream]
base_url = "https://slipstream.mara.com"
fee_endpoint = "/api/rates"
submit_endpoint = "/api/transactions"
"#;
        let config = parse_config(text, "test.toml").unwrap();
        assert_eq!(config.slipstream.client_code, "");
        assert_eq!(config.slipstream.request_timeout_secs, 30);
        assert_eq!(
            config.broadcast_api.listen_addr,
            "127.0.0.1:3010".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(config.broadcast_api.fee_poll_secs, Duration::from_secs(60));
        assert_eq!(
            config.broadcast_api.rate_limit_window_secs,
            Duration::from_secs(600)
        );
        assert_eq!(config.broadcast_api.rate_limit_max_tx, 100);
        assert_eq!(config.broadcast_api.max_payload_bytes, 1_048_576);
    }

    #[test]
    fn rejects_invalid_toml() {
        let err = parse_config("not valid toml {{{", "test.toml").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn parse_error_does_not_include_config_contents() {
        let secret = "DISTINCTIVE_SECRET_VALUE";
        let err = parse_config(
            &format!("[slipstream]\nrequest_timeout_secs = \"{secret}\""),
            "secret.toml",
        )
        .expect_err("secret in a numeric field must fail parsing");

        assert!(!err.to_string().contains(secret));
        assert!(!format!("{err:?}").contains(secret));
    }

    #[test]
    fn rejects_config_missing_required_slipstream_table() {
        let err = parse_config("[broadcast_api]\n", "test.toml").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn rejects_invalid_listen_addr() {
        let text = r#"
[slipstream]
base_url = "https://slipstream.mara.com"
fee_endpoint = "/api/rates"
submit_endpoint = "/api/transactions"

[broadcast_api]
listen_addr = "not an address"
"#;
        let err = parse_config(text, "test.toml").unwrap_err();
        assert!(matches!(err, ConfigError::InvalidListenAddr { .. }));
    }

    #[test]
    fn missing_file_is_a_read_error() {
        let err = load_config("/nonexistent/path/to/outofband-config.toml").unwrap_err();
        assert!(matches!(err, ConfigError::Read { .. }));
    }
}
