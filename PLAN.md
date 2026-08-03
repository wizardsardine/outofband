# Outofband — Plan

Single-page web service to broadcast Bitcoin transactions through the MARA
Slipstream direct-submission service. Full Rust stack — axum backend,
Yew/WASM frontend, no hand-written JavaScript — with per-IP rate limiting,
batch submission via drag-and-drop (single files, multiple files, or
tar/tar.gz/zip archives unpacked in the browser), and one-command deployment
on a fresh Debian server behind nginx with Let's Encrypt TLS.

This document is self-contained: it specifies the architecture, every
convention, the wire formats, and the deployment tooling in enough detail to
implement the project from this file alone.

## Summary

The rest of this document (sections 0–8) is the implementation
specification — reference material to work from, not to read through.
This section is the whole design.

### What it is

A one-page site that relays Bitcoin transactions to MARA Slipstream,
which mines them directly instead of gossiping them through the public
mempool. It exists for the case where a key is known to be compromised —
predictable RNG — and a normal broadcast would lose the race to whoever
else can spend those coins.

Full Rust: axum backend, Yew/WASM frontend, one shared `tx-core` crate,
no hand-written JavaScript, and no system C library linked anywhere
(`reqwest` uses `rustls`, not OpenSSL). The graph does vendor C in two
places — libsecp256k1 and rustls' crypto provider — see section 3.

### Flow

```
browser
  paste box │ dropped files │ dropped .tar/.tar.gz/.zip
      └─> unpack.rs   archives extracted in-browser, magic-byte sniffed,
      │               entries sorted lexicographically
      └─> tx-core     detect → parse → analyze → finalize → extract hex
                      → vsize, fee rate (when derivable), txid
      └─> queue row   one per transaction, nothing sent yet

  "Broadcast all"  sequential, one HTTP call per tx, in queue order
      │
      ▼
server (axum, 127.0.0.1:3010, behind nginx)
  POST /broadcast {tx_hex}
      well-formed transaction with inputs and outputs?
      → atomically reserve rate-limit slot → submit
      │
      ▼
MARA Slipstream
  {client_code, tx_hex} → accepted / rejected (fee floor is theirs alone)
```

### The five decisions everything else follows from

1. **One transaction per request.** The API accepts `{tx_hex}` and
   nothing else: no multipart, no file upload, no archive handling, no
   PSBT path, no decompression. Batching is a frontend loop. The server's
   entire input surface is one bounded JSON string, so the
   decompression-amplification DoS class does not exist here rather than
   being bounded. Cost: scripted callers loop instead of posting an
   archive.
2. **All analysis happens in the browser.** `tx-core` compiles to wasm
   and does format detection, PSBT analysis, finalization, extraction,
   vsize, fee and txid. The server only ever sees finalized hex. Queueing
   500 items causes zero API traffic; there is no check/dry-run endpoint.
3. **Fee rates are displayed, never enforced.** Nothing in this service
   refuses a submission over its fee — not the browser, not the server.
   A below-floor transaction is submitted and bounced, because that costs
   one round trip and Slipstream's answer is more authoritative than a
   comparison against a floor we cached up to a minute ago. MARA's
   no-recourse warning is about transactions that *succeed*; withholding
   never protected against those.
4. **Nothing leaves the queue.** Rejected, rate-limited and failed rows
   keep their place, their analysis, their reason and their txid, and can
   be retried individually or by pressing Broadcast again. Every row
   carries a locally derived txid from the moment it is queued, so a user
   whose submission bounced can still identify the transaction they
   built. Only `×` or "Clear queue" removes a row.
5. **The client code never reaches the browser.** It lives only in
   `/etc/outofband/config.toml` (root:outofband, 640), is injected by
   `slipstream-client` into every submission, and is redacted from every
   error, log line and response. This — plus CORS and the ability to
   rate-limit — is why the browser does not call MARA directly.

### Crates

| crate                | kind             | role                                |
|----------------------|------------------|-------------------------------------|
| `tx-core`            | lib, native+wasm | detect, analyze, finalize, extract, vsize, fee, txid |
| `slipstream-client`  | lib              | MARA client; sole holder of the client code |
| `broadcast-api`      | bin (axum)       | 3 routes, fee cache, per-IP limiter |
| `broadcast-frontend` | bin (Yew, trunk) | page, queue, `unpack.rs`, broadcast loop |

`tx-core` building for `wasm32-unknown-unknown` is a hard requirement, not
an aspiration: it is what makes the number the user sees and the
transaction the server relays come from one implementation.

### HTTP API

| Method | Path         | Body / response                             |
|--------|--------------|---------------------------------------------|
| GET    | `/fee`       | cached floor + age + `stale` flag           |
| POST   | `/broadcast` | `{tx_hex}` → `{txid, vsize, status, error}` |
| GET    | `/health`    | liveness                                    |

`status` is `submitted|rejected|invalid`. 400 unparseable hex, 413 over
`max_payload_bytes` of transaction hex (1 MiB default), 429 when rate limited, 502 Slipstream
unreachable; application-level 429 responses include `retry_after_secs`,
while nginx can return a non-JSON 429. 200 whenever a submission was attempted and
answered, including a fee-too-low rejection surfaced verbatim.

`GET /fee` is served from a background-polled cache, so anonymous page
loads never fan out into requests against MARA. Rate limiting is a per-IP
sliding window (100 submissions / 600 s by default), keyed off an
`X-Real-IP` value overwritten by nginx. Proxy headers are trusted only when
the socket peer is localhost. Localhost is exempt.

### Frontend

- Input: paste (one entry per non-blank line, `#` comments skipped),
  files, or `.tar`/`.tar.gz`/`.tgz`/`.zip` archives — all sniffed by
  content, never by extension.
- Archive entries and files sort lexicographically and submit strictly
  sequentially in that order, so naming files `01-parent`, `02-child`
  broadcasts a dependent chain correctly. This ordering is guaranteed.
- Extraction is in-browser under self-protection limits (256 MiB budget,
  1,000 entries, no nesting). A hostile archive can at worst disrupt the
  tab of the person who dropped it; the server never sees it.
- A PSBT that cannot be finalized never enters the queue — one modal per
  load lists every refused item. It is the only thing the UI refuses on
  the user's behalf, and only because an unsigned transaction cannot
  succeed under any fee.
- Each row: format chip, vsize, txid, fee rate against the live floor,
  status, a colored note card, and remove/retry controls. Fee-unknown
  rows (raw transactions, and PSBTs missing UTXO data) expose a
  "total input value" field that derives the rate — informational only.
- Broadcast: sequential, awaited. A 429 pauses with a countdown and
  resumes; a rejection marks the row and continues; a network failure
  pauses with Retry.
- `UI_MOCKUP.html` at the repo root is the authoritative visual
  reference. Section 5 transcribes it and lists the eight deliberate
  departures; anything not on that list is a detail to reproduce.

### Configuration and deployment

Everything lives in `/etc/outofband/config.toml`: Slipstream `base_url`,
the two endpoint paths, `client_code`, timeout; then `listen_addr`,
`fee_poll_secs`, `rate_limit_window_secs`, `rate_limit_max_tx`,
`max_payload_bytes`. Endpoint paths are config, not code, so a change at
MARA is an edit plus a restart.

Three bash scripts under `deploy/`, each running locally with no argument
or against `user@host` by rsync + re-exec: `install.sh` (apt deps,
toolchain, user, build, systemd, nginx, health check, optional
`--domain/--email` certbot), `update.sh` (code only — never touches the
live config or certs), `clean.sh` (full teardown, confirming before
deleting the config that holds the client code).

### Slipstream, in one paragraph

Transcribed from the live API on 2026-08-02 (section 2). Fee numbers come
from `GET /api/rates`, uncredentialed, so the page works on a fresh deploy
before any client code exists; the headline figure is `effective_rate`
(4.0 sat/vB at time of writing), the rate at which a transaction actually
gets mined, not `submit_fee_rate` (2.0), which only buys admission to a
mempool that may never mine it. Submission is
`POST /api/transactions {client_code, tx_hex}` and **does** require the
client code, despite the spec marking it optional. The response is
`{status, message}` with **no txid**, which is why deriving the txid
locally is load-bearing. A packages endpoint exists for CPFP; this project
does not use it, so CPFP is out of scope.

### Accepted limitations

No fee validation anywhere: a scripted caller can submit any well-formed
transaction encoding with inputs and outputs and receive MARA's rejection.
Slipstream performs the actual transaction validation. The displayed floor is
cached and can lag the real threshold by up to the poll interval, so a
comfortable-looking rate can still bounce. Rejections consume the per-IP
budget like any other submission, which is the price of not gating. Queue
behavior is frontend policy, not a server guarantee, since the server
sees one transaction at a time. The default 1 MiB cap excludes the
largest non-standard transactions (~8 MiB hex) unless raised. The service
is an open relay by design.

## 0. Core architectural decision — one call per transaction

The API is deliberately minimal: the server accepts exactly one transaction
per request, as a small JSON body, and nothing else. Batching is a frontend
behavior — the browser unpacks archives, expands multi-line files, analyzes
and finalizes everything locally, and loops over the queued transactions,
making one `/broadcast` call each, sequentially.

This decision resolves a real design tension and is worth recording. Two
earlier iterations were considered. Server-side batch intake
(multipart uploads, server-side archive unpacking) supports scripted
archive submission, but creates a decompression-amplification DoS surface
that must then be bounded with per-entry/total budgets and a concurrency
semaphore. Browser-side unpacking with server-side multipart removes that
surface but duplicates limit enforcement and still needs multipart handling.
One-call-per-tx with a small JSON payload makes the amplification surface
not bounded but *nonexistent*: the server performs no decompression, parses
no archives, handles no multipart — its entire input is one bounded JSON
string per request. The costs, accepted deliberately: scripted callers
submit one transaction per request rather than posting archives (a shell
loop), and batch atomicity is a frontend policy rather than a server
guarantee — which is honest, since direct API callers were never bound by
it anyway.

## 1. What the service does

The user opens a single dark-themed page (full UI specification in
section 5, matching the approved mockup). A hero shows Slipstream's live
`effective_rate` — "the floor" — refreshed periodically, with the rule
stated plainly: anything below it will not be mined.
Below, the user loads transactions two ways: pasting into a text area
(a single PSBT in base64 or raw transaction in hex — or several, one per
line) and dropping/choosing files — single files, several at once, or
archives (`.tar`, `.tar.gz`/`.tgz`, `.zip`), unpacked in the browser
(section 5). Everything loaded lands in a queue table where each item is
analyzed locally and shows its format, vsize, txid, fee rate when
derivable, and how that rate compares to the floor. "Broadcast all" then
submits every queued transaction, one call each, sequentially, and per-item
results (accepted, or the precise rejection reason) appear in place.
Nothing is removed from the queue by broadcasting: a rejected or failed
transaction stays visible, with its txid, and can be retried.

Accepted file contents, detected per file by content with no reliance on
extensions (the picker advertises `.txt .psbt .txn .hex .raw .tar .tar.gz
.tgz .zip`): raw binary PSBT (leading magic `psbt\xff`), base64-encoded
PSBT text, hex transaction text, and raw binary transaction
(consensus-deserializes as a `bitcoin::Transaction`). A text file — or the
paste box — may contain several entries separated by newlines, each
independently base64-PSBT or hex; blank lines and lines starting with `#`
are ignored. Archive entries and files are ordered lexicographically by
name, and the frontend submits items strictly sequentially in that order —
this ordering is documented and guaranteed, so a dependent chain can be
submitted correctly by naming files `01-parent`, `02-child`, and so on.
This works only where each transaction clears the floor on its own:
Slipstream judges every submission independently, so a CPFP pair whose
parent deliberately underpays cannot be rescued by ordering and is out of
scope for this service (section 2). Nested archives are rejected.

Fee handling per item is **advisory, never a gate**. For PSBTs carrying
`witness_utxo`/`non_witness_utxo` data, the absolute fee and effective fee
rate are computed exactly, and the row is labeled Ready (rate ≥ floor) or
Below floor. A PSBT missing UTXO data for some inputs, and every raw
transaction (previous outputs unknown), shows Fee unknown — and the row
exposes a "total input value" field where the user can enter the summed
input amounts, from which the fee rate is derived. All three labels are
information the user acts on; none of them stops a submission.
"Broadcast all" submits every queued transaction, including Below-floor
and Fee-unknown ones.

This is a deliberate reversal of an earlier, stricter design in which only
items with a known rate at or above the floor were ever sent. Gating was
the wrong instrument. A fee-too-low submission is rejected by Slipstream
at no cost and no risk — the transaction is not mined, not published, not
consumed; the round trip is the entire penalty, and the rejection message
is more authoritative than our own comparison against a cached floor that
may already have moved. MARA's no-support, no-recourse warning is about
transactions that *succeed* — an incorrect fee that gets mined, a
malformed spend that cannot be undone — and refusing to submit never
protected against that. So the browser computes the fee rate, shows it
plainly against the live floor, and lets the user decide; what it refuses
to do is decide for them.

The consequences are recorded honestly: below-floor items consume the
per-IP rate-limit budget for submissions that predictably bounce
(section 4), and the queue's counts of Below-floor and Fee-unknown items
are advisory rather than a work list to clear before broadcasting. (The
user-entered input total is unverifiable by us and now gates nothing at
all — it exists so the user can see a rate before committing. The API
trusts the caller, and Slipstream remains the final validator.)

Because every queued item has a locally parsed or finalized transaction,
its txid is known before anything is broadcast. `tx-core` computes it from
the transaction. The queue therefore shows a txid for every row from the
moment it is queued, independent of broadcast outcome, so a transaction
whose submission is rejected or never attempted is still identifiable and
recoverable by the user rather than a row they can only delete.

All parsing, PSBT finalization, and extraction happen in the browser via
`tx-core` compiled to wasm. When an item is queued, `tx-core` detects the
format, decodes it, validates its UTXO maps, analyzes its fee when
derivable, and finalizes it if needed. When every UTXO is available,
miniscript's checked extractor validates each finalized input without
applying rust-bitcoin's fee-rate limit. UTXO transaction IDs, output indexes, and duplicate witness and
non-witness UTXO descriptions must agree before either analysis or
finalization proceeds. What the frontend holds for every queued item is
the finalized transaction hex and its txid; that hex is the only thing
ever sent to the server. An already-finalized PSBT missing UTXO data can be
extracted at the same trust level as a raw transaction, with an explicit
warning that its fee and final scripts were not validated. Other PSBTs that
cannot be finalized never leave the browser. The modal lists every refused
item and its actual finalization reason while the rest of the load queues
normally.
Binary uploads are handled the same way — bytes in, analyzed item out —
so no conversion rules burden the user.

The API consequently accepts exactly one thing: finalized raw transaction
hex, one transaction per request. Server-side handling is minimal and
mechanical: the hex must decode as a well-formed `bitcoin::Transaction`,
have non-empty inputs and outputs, and fit the payload cap. This does not
validate scripts or chain context; Slipstream performs that validation.
There is no PSBT code path, no format detection, and no
fee computation on the server (fee cannot be derived from a raw
transaction without its previous outputs, and PSBTs, where it could be,
never arrive). Fee-floor enforcement exists nowhere in this service —
neither browser nor server refuses a submission over its fee rate;
Slipstream is the sole and final authority on the floor, and the server is
a validating, rate-limited relay in front of it. Scripted API callers
submit finalized hex
(a PSBT is finalized locally first, e.g. with `bitcoin-cli finalizepsbt`
or by pasting it into the web page); this is documented in the README.

## 2. About the Slipstream API

Slipstream is MARA Pool's direct transaction submission service: it accepts
consensus-valid transactions (including non-standard ones that public
mempools reject) provided they meet MARA's minimum fee threshold, and mines
them via MARA Pool. That threshold is dynamic — MARA adjusts it with network
conditions — which is exactly why the page must fetch it live rather than
hard-code it.

Everything below was transcribed from the OpenAPI 3.1 spec at
`https://slipstream.mara.com/docs/openapi.json` ("Slipstream Beta API",
spec version `0.1.0-beta`, service reporting `1.2.0`, chain `bitcoin`) and
**verified against the live API on 2026-08-02**. Where the spec and the
deployed service disagree, the service wins and the disagreement is noted.

### Endpoints

| Method | Path                       | Credential | Purpose                          |
|--------|----------------------------|------------|----------------------------------|
| GET    | `/api/rates`               | none       | the fee numbers (see below)      |
| GET    | `/api/system`              | none       | version, height, floor, features |
| GET    | `/api/block-info`          | none       | height, `fee_rate`, `submit_fee_rate` |
| GET    | `/api/transactions/status` | none       | confirmation state by `tx_id`    |
| POST   | `/api/transactions`        | client code| submit one transaction           |
| POST   | `/api/transactions/packages` | client code | 2–25 txs as a package          |
| POST   | `/api/mempool/tests`       | **Authorization header** | testmempoolaccept  |

This project uses exactly two: `GET /api/rates` and
`POST /api/transactions`.

### Credentials

There are two, and neither appears in the spec's `security` block — the
spec declares no authentication at all, which is simply wrong about the
deployed service.

**The client code** is required for submission. The spec annotates
`client_code` as an optional field "used to apply a reduced fee rate
threshold", but a live `POST /api/transactions` carrying only `tx_hex`
returns `400`:

```json
{"status":"error","message":"Client codes are currently required to submit
transactions. Contact us at foundation@mara.com for more information."}
```

So it is both a credential and a discount code. It lives in
`/etc/outofband/config.toml` only, is injected server-side into every
submission, and never reaches the frontend, logs, or version control.

**An `Authorization` header** gates `POST /api/mempool/tests`, which
returns `401 {"is_success":false,"message":"Missing or invalid
Authorization header"}` without one. That credential is undocumented and
we do not have it, so the dry-run endpoint is unavailable to this project
regardless of design preference.

Everything the frontend needs to render on load — the fee numbers — is
uncredentialed. The fee poller therefore works on a fresh deploy before
any client code is configured, and only broadcasting requires the secret.

### The fee numbers

`GET /api/rates` (optionally `?client_code=…`, which applies volume
discounts to the result) returns seven fields. Live values on 2026-08-02:

```json
{"market_rate":2.0, "multiplier":2.0, "multiplier_discount_percent":0,
 "discounted_multiplier":2.0, "submit_fee_rate":2.0,
 "slipstream_rate":4.0, "effective_rate":4.0}
```

Two of these are thresholds, and they are not the same thing:

- `submit_fee_rate` (= `1.0 × discounted_multiplier`) is the **admission**
  line. Below it, submission is rejected outright.
- `effective_rate` (= `market_rate × discounted_multiplier`) is the
  **mined** line — "the rate at which a typical slipstream transaction
  would currently be mined". MARA states the minimum as "the higher of
  either 2x the current mempool priority fee rate or 2 sats/vByte".

Between the two lies a trap: a transaction paying 2.5 sat/vB today is
*accepted* — the caller gets `"status":"success"` — and then sits in the
private mempool without ever being mined, because "admission into the
mempool does not guarantee a transaction will ever be mined. Submitted
slipstream transactions are mined on a best-effort basis."

**The UI shows `effective_rate` and nothing else.** For a tool whose users
are racing a thief, the number that matters is the one that gets a
transaction into a block; a headline figure of `submit_fee_rate` would
report success to someone who has not actually won. `/api/system`'s
`slipstream_fee_rate_floor` carries the same value as `effective_rate` and
is a viable fallback source if `/api/rates` ever fails.

### Submission

```json
POST /api/transactions
{"client_code": "<code>", "tx_hex": "<raw_transaction_hex_string>"}
```

`skip_mempool_submission: bool` also exists (validate and store without
relaying); this project never sets it. The response, on both `200` and
`400`, is:

```json
{"status": "success|error", "message": "<free text>"}
```

**There is no txid field.** The spec says `message` "can include context
on success or failure, such as transaction ID or error description" — free
prose, not a contract. This is why `tx-core` deriving the txid locally
(section 1) is load-bearing rather than a convenience: it is the only
reliable source of the identifier. `message` is surfaced verbatim to the
user as the outcome reason and never parsed for meaning.

Note the two distinct error shapes: submission failures return
`{status, message}`, while `/api/rates` and `/api/transactions/status`
failures return `{is_success, message}`. `slipstream-client` handles both.

### Packages, and the CPFP limit

`POST /api/transactions/packages` accepts 2–25 hexes with parents ordered
before the child, and is how MARA supports CPFP: each transaction is
checked for fee thresholds as part of the package rather than alone.

**This project does not use it.** Consequently CPFP does not work here: a
deliberately low-fee parent submitted on its own is rejected on its own
fee rate, and sequential ordering cannot rescue it. Ordering guarantees
(section 1) still hold for chains where each transaction independently
clears the floor — a parent funding a child — but a package whose parent
depends on its child's fee must be submitted through MARA directly. This
is stated in the README rather than left for a user to discover. Adding a
package path would mean our own API accepting an array, which is the shape
section 0 designed out; it is a documented follow-up, not a first release.

MARA also states plainly that Slipstream operates without technical or
customer support and at the submitter's own risk: an incorrectly built
transaction or an incorrect fee cannot be helped after the fact. That
warning is a design input, not just a disclaimer — it is why every queued
item is analyzed and its exact fee rate shown against the live floor
before the user commits (sections 1 and 5), and why the backend checks that
what it relays is a well-formed transaction encoding with inputs and outputs
(section 4). What the warning does *not* justify is refusing
to submit on the user's behalf: the mistakes MARA cannot help with are
mistakes that get mined, and a rejected submission costs a round trip and
nothing else. Analysis informs the decision; it does not make it
(section 1).

### Client crate

Endpoint paths stay in config rather than code, so a change on MARA's side
is a config edit plus a restart rather than a rebuild — which matters more
than usual for an API self-described as beta. The `slipstream-client`
crate (section 3) wraps the two endpoints:

```rust
pub struct SlipstreamClient { /* reqwest + base_url + endpoints + client_code */ }

pub struct FeeInfo {
    pub effective_rate_sat_vb: f64,   // GET /api/rates -> effective_rate
    pub fetched_at: DateTime<Utc>,
}

impl SlipstreamClient {
    pub async fn rates(&self) -> Result<FeeInfo, SlipstreamError>;
    pub async fn submit_tx(&self, tx_hex: &str) -> Result<SubmitResult, SlipstreamError>;
}
```

`rates()` appends `?client_code=…` when one is configured, so the returned
`effective_rate` reflects any volume discount; it works without one.
`submit_tx` builds the `{client_code, tx_hex}` body itself, taking the code
from its config at construction; no other module ever touches the
credential. `SubmitResult` carries Slipstream's `status` and its `message`
verbatim.

`SlipstreamError` distinguishes network failure, HTTP error status,
missing/invalid client code, and a rejection carrying MARA's message, so
the API layer can map each to a proper HTTP status and user-facing
message — with the guarantee that the client code is never echoed back in
any error, log line, or response. Note that the client cannot classify
*why* a submission was rejected: `{status:"error", message:"…"}` is prose,
so fee-too-low and consensus failure are the same variant, distinguished
only by the text shown to the user. Keeping the credential server-side is
one of the reasons the browser must not call Slipstream directly, along
with CORS and the ability to rate-limit abusive callers ourselves.

## 3. Repository layout and workspace conventions

The project is a single Cargo workspace, one crate per concern, with all
deployment material under `deploy/`:

```
outofband/
├── Cargo.toml                      # workspace: members, shared deps, shared lints
├── Cargo.lock                      # committed
├── .justfile                       # developer commands (see section 6)
├── .gitignore                      # target/, dist/, dev-config.toml
├── README.md                       # user-facing: deploy, config, operations
├── CLAUDE.md                       # working notes for AI-assisted development
├── UI_MOCKUP.html                  # authoritative UI reference (see section 5)
├── crates/
│   ├── slipstream-client/          # HTTP client for MARA Slipstream (lib)
│   │   └── src/lib.rs
│   ├── tx-core/                    # shared lib: detection, PSBT analysis/finalize,
│   │   └── src/lib.rs              #   vsize + fee math; compiles native AND wasm
│   ├── broadcast-api/              # axum REST backend (bin)
│   │   └── src/
│   │       ├── main.rs             # config loading, router, background fee poller
│   │       ├── routes.rs           # handlers, AppState, FeeCache
│   │       └── rate_limit.rs       # per-IP sliding-window limiter (see section 4)
│   └── broadcast-frontend/         # Yew CSR app built with trunk (bin, wasm)
│       ├── Trunk.toml
│       ├── index.html              # single HTML shell + embedded CSS + bundled fonts
│       └── src/
│           ├── main.rs             # page, state, queue, sequential broadcast loop
│           └── unpack.rs           # in-browser tar/tgz/zip extraction + item expansion
└── deploy/
    ├── install.sh                  # full bootstrap of a fresh Debian server
    ├── update.sh                   # code-only redeploy
    ├── clean.sh                    # remove everything install.sh created
    ├── config.toml                 # template for /etc/outofband/config.toml
    ├── nginx/outofband.conf        # port-80 server wrapper
    ├── nginx/outofband-app.conf    # managed routes, limits, and cache policy
    ├── nginx/outofband-security-headers.conf
    └── systemd/broadcast-api.service
```

Workspace `Cargo.toml` conventions: `resolver = "3"` (edition 2024's
resolver; it must be set explicitly at the workspace root or cargo warns);
`[workspace.package]`
sets `version`, `edition = "2024"`, `rust-version = "1.88"`, inherited by
every crate via `version.workspace = true` etc.; all dependency versions are
declared once under `[workspace.dependencies]` and referenced from crates
with `foo.workspace = true`; `default-members` lists only the native crates
so a bare `cargo build`/`cargo test` never tries to build the wasm frontend
for the host target. Shared lint policy, inherited by every crate through
`[lints] workspace = true`:

```toml
[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
complexity  = { level = "deny", priority = -1 }
perf        = { level = "warn", priority = -1 }
```

Workspace dependencies — backend and client: `axum 0.8` (feature `macros`),
`tokio 1` (features `full`), `serde 1` (derive), `serde_json 1`,
`reqwest 0.12` (`default-features = false`, features `json` and
`rustls-tls` — never the default `native-tls`, which would link the
system OpenSSL and drag `libssl-dev` into the deploy prerequisites),
`toml 0.8`, `base64 0.22`, `bitcoin 0.32`
(`default-features = false`, features `std`, `base64` for PSBT
serialization), `chrono 0.4` (serde). `tx-core` depends on `bitcoin`,
`miniscript 12` (`no-std`-capable, pure Rust, tracking `bitcoin 0.32`),
`base64` and `serde`. `miniscript` is required rather than optional:
`bitcoin` alone can `extract_tx()` an already-finalized PSBT but cannot
*finalize* one — the finalizer lives in `miniscript::psbt::PsbtExt`, and
finalizing a signed-but-unfinalized PSBT is exactly what section 1
promises. It compiles for both the host and
`wasm32-unknown-unknown` — this dual-target property is a hard requirement
and is enforced by CI/justfile (`cargo build -p tx-core --target
wasm32-unknown-unknown`). Frontend: `tx-core`, `yew 0.21` (feature `csr`),
`gloo-net 0.6`, `gloo-timers 0.3`, `gloo-file` (async file reads),
`wasm-bindgen-futures 0.4`, `web-sys 0.3` with the features drag-and-drop
needs (`HtmlTextAreaElement`, `HtmlInputElement`, `DragEvent`,
`DataTransfer`, `FileList`, `File`), and — because archive extraction runs
in the browser — the archive crates compiled to wasm: `tar`, `flate2` (with
the `rust_backend`/miniz_oxide backend), and `zip`
(`default-features = false`, feature `deflate` only). All three are pure
Rust and build cleanly for `wasm32-unknown-unknown`.

On "no C/C++", stated precisely rather than as a slogan, because the
convenient version of this claim is false. **C reaches the browser.**
`bitcoin 0.32` requires `secp256k1 0.29`, which vendors libsecp256k1 —
C, built with `cc` — and it compiles to wasm along with everything else,
so the shipped `.wasm` contains C-derived code. The backend additionally
gets C and assembly from `rustls`'s crypto provider (`ring` or
`aws-lc-rs`). Both are vendored and compiled from source as part of the
build, not linked from the system.

What is actually true, and what the README should say: this workspace
contains no C or C++ source of its own; nothing links a *system* C
library (which is why `libssl-dev` is not an install prerequisite); the
archive stack (`tar`, `flate2`/miniz_oxide, `zip`) is pure Rust; and the
only toolchain requirement beyond rustc is a working `cc`, which
`build-essential` provides. A literally C-free dependency graph is not
achievable for a project that validates Bitcoin signatures, and claiming
one would be a lie. Supported archive formats are exactly tar,
tar.gz/tgz, and zip. The IBM Plex Sans and IBM Plex Mono woff2 files ship
as trunk assets in the frontend crate — the exact weights the mockup
declares (Sans 200/300/400/400-italic/500/600/700, Mono
300/400/500/600/700), extractable from `UI_MOCKUP.html`'s bundle so the
rendered page matches it byte for byte.

## 4. Backend: `broadcast-api`

Axum service listening on `127.0.0.1:3010`, configured entirely from
`/etc/outofband/config.toml`. The config path can be overridden as the first
CLI argument (`broadcast-api /path/to/config.toml`), which is how the
systemd unit invokes it; with no argument it falls back to the default path.
On startup: parse config, construct the `SlipstreamClient`, spawn the
fee-poller task, build the router with a shared `AppState` (client + fee
cache + rate limiter, all cheaply clonable via `Arc`), and serve.

Routes, at the root — there are only three, and they are named, so a
prefix buys nothing:

| Method | Path         | Purpose                                        |
|--------|--------------|------------------------------------------------|
| GET    | `/fee`       | Current Slipstream minimum fee rate (cached)   |
| POST   | `/broadcast` | Submit ONE finalized transaction (hex)         |
| GET    | `/health`    | Liveness for monitoring and install.sh checks  |

The consequence is that nginx cannot proxy by prefix and must match the
three paths explicitly (section 6), and that the API and the static-asset
namespaces now share a root: a future frontend asset or route named
`/fee`, `/broadcast` or `/health` would collide with the API. With a
fixed three-route surface that is a documented constraint, not a risk.

There is no check/dry-run endpoint: all analysis happens in the browser via
`tx-core` in wasm (section 5), so queueing five hundred items causes zero
API traffic until Broadcast, and scripted callers who can produce
transaction hex can equally decode it locally before submitting.

### Fee cache

`GET /fee` is served from a `FeeCache` — an
`Arc<RwLock<Option<FeeInfo>>>` refreshed by a background tokio task every
`fee_poll_secs` (default 60) calling `SlipstreamClient::rates()`. Handlers
only ever read the cache; anonymous page loads never fan out into requests
against MARA. The response includes the fee rate, the age of the cached
value, and a `stale: bool` flag set when the last 3 refresh attempts failed,
so the frontend can warn rather than display a dead number.

### Broadcast

`POST /broadcast` takes `{ "tx_hex": "<finalized_transaction_hex>" }` —
one transaction per request, hex only; the field name deliberately mirrors
Slipstream's own. There is no multipart, no file upload, no archive
handling, and no PSBT code path anywhere in the backend. The request body
accepts up to `max_payload_bytes` of transaction hex (default 1 MiB). The
axum body limit also allows the fixed JSON framing, and nginx permits a
coarser whole-body ceiling in front (section 6). Note for
operators: a maximally large non-standard transaction — the kind Slipstream
exists for — can reach ~4 MB serialized, ~8 MiB as hex; if such
transactions are expected, raise `max_payload_bytes` (and the templated
nginx value) to 8 MiB. Without any decompression step, a larger cap costs
only what the bytes themselves cost.

Pipeline: trim and decode a well-formed `bitcoin::Transaction`, reject empty
inputs or outputs, atomically check and record the rate-limit slot, then call
`slipstream_client.submit_tx()`. Invalid encodings do not consume the
allowance. The server
performs no fee computation — a raw transaction's fee is underivable
without its previous outputs, and PSBTs, where fees could be derived, never
reach the server — and the browser does not gate on fee either
(section 1), so Slipstream alone enforces the floor, with the server as a
validating, rate-limited relay in front of it. A consequence worth stating
for operators: because the frontend submits below-floor items rather than
withholding them, a share of each IP's rate-limit budget is spent on
submissions that predictably bounce.

Response body:
`{ "txid": "...", "vsize": n,
"status": "submitted|rejected|invalid",
"error": <string|null> }`.
HTTP statuses: 400 not valid transaction hex, 413 payload over
`max_payload_bytes`, 429 rate limited, and 502 Slipstream unreachable.
Application-level 429 responses carry an
`{ "error": ..., "retry_after_secs": n }` body; nginx can return a non-JSON
429 before the application. The frontend retries either form. A 200 is used
when Slipstream answers the submission, with the outcome in `status`
(including a fee-too-low rejection surfaced verbatim per item).

Parsing lives in the shared `tx-core` crate — format detection, PSBT UTXO
validation and fee derivation, checked finalization and extraction, vsize,
and fee-rate math. The frontend
compiles all of it to wasm; the backend uses only the hex-decode/validate
subset, so the number the user sees in the queue and the transaction the
server relays come from the same code. (The mockup approximates PSBT vsize
with a JavaScript estimator; the real implementation replaces this with
exact figures from the `bitcoin` crate, which builds cleanly for
`wasm32-unknown-unknown`.) `tx-core` is
unit-testable without the network: format detection, the PSBT finalization
matrix (already finalized / finalizable / incomplete), vsize and fee
computation, and fee-rate comparison all get table-driven tests with
fixture transactions. Native tests exercise the full crate, and the wasm
build plus frontend browser tests cover its browser integration.

### Rate limiting

Application-level, in-process, per-IP. Because the frontend makes one call
per transaction, a one-shot-per-window limiter would break batches (the
second item would wait out the window), so the limiter is a sliding-window
counter: each IP may attempt at most `rate_limit_max_tx` submissions
(default 100) within any rolling `rate_limit_window_secs` window (default
600). This bounds per-IP volume directly — more honestly than the earlier
one-shot-plus-batch-cap combination did.

```rust
#[derive(Clone)]
pub struct RateLimiter {
    window: Duration,                                   // from config
    max_tx: usize,                                      // from config
    requests: Arc<Mutex<HashMap<IpAddr, VecDeque<Instant>>>>,
}
```

`check_and_record(ip) -> Result<(), Duration>` prunes expired entries for
all addresses, then checks and records one slot while holding the same lock.
If the deque already holds `max_tx` entries, it returns the time until the
oldest entry leaves the window. The handler rounds that duration up for
`retry_after_secs`. Requests failing transaction decoding never call it and
do not consume the allowance. Localhost is always exempt (keeps local testing and
the install health check painless). Unit tests: first `max_tx` allowed and
the next blocked with a sane remaining duration, allowance recovering as old
entries expire, distinct IPs independent, localhost exempt.

Proxy headers are considered only when the socket peer is localhost.
`X-Real-IP`, which nginx overwrites with `$remote_addr`, is preferred;
nginx's overwritten `X-Forwarded-For` is the fallback. A non-local peer's
socket address always wins over supplied headers.

### Configuration

`/etc/outofband/config.toml` template (shipped as `deploy/config.toml`):

```toml
[slipstream]
base_url = "https://slipstream.mara.com"
fee_endpoint = "/api/rates"             # -> effective_rate, no credential
submit_endpoint = "/api/transactions"   # -> {status, message}
client_code = ""                        # secret — root:outofband, mode 640
                                        # required to submit; rates work without
request_timeout_secs = 30

[broadcast_api]
listen_addr = "127.0.0.1:3010"
fee_poll_secs = 60
rate_limit_window_secs = 600
rate_limit_max_tx = 100
max_payload_bytes = 1048576          # Maximum transaction-hex length. nginx
                                     # adds JSON framing and rounds up to MiB.
```

## 5. Frontend: `broadcast-frontend`

Yew 0.21 CSR application built with trunk (`trunk build --release`) into
static `index.html` + wasm-bindgen `.js` glue + `.wasm`, served by nginx
from `/var/www/outofband/`. No hand-written JavaScript; the only JS shipped
is the loader trunk generates. State is a handful of `use_state` hooks;
requests go through `gloo-net`; item analysis calls `tx-core` directly in
wasm.

`UI_MOCKUP.html` at the repository root is the **authoritative visual
reference**. Where this section and the mockup disagree on layout, color,
spacing, radius, sizing or copy, the mockup wins and this section is to be
corrected — with the sole exception of the five departures enumerated at
the end of this section, which exist because the mockup is a static prop
with no backend and therefore both asserts things about the architecture
that are untrue of the built service and simulates behavior it does not
implement. Everything else below is transcribed from it.

The mockup is a bundler-packed React page: its markup and stylesheet live
in a JSON-escaped `__bundler/template` block, and the fonts and design-
system bundle in a gzip+base64 `__bundler/manifest` block, both extractable
with a few lines of Python (`json.loads` the template,
`gzip.decompress(base64.b64decode(...))` each manifest entry). The
extracted template — inline styles throughout, one `DCLogic` component
class — is the source of every value in this section, and the Wizardsardine
design-token sheet it embeds (`variables.css` + `font.css` from
wizardsardine.com) is the origin of the palette.

### Design tokens

Dark theme throughout. Background `#000000`; cards `#0c0c0c` (nested
`#060606`); borders `#2a2a2a` (strong) and `#1f1f1f`/`#1a1a1a` (hairline);
text `#f4f4f4` (primary), `#e6e6e6` (field text), `#a1a1a1` (secondary),
`#909090` (body copy in cards), `#7b7b7b`/`#6a6a6a` (muted), `#4a4a4a`
(disabled), `#454545` (placeholder). Accent teal `#5fe7e4`/`#61ffe1` for
primary actions, success, and hovers; red `#ef445f` for errors/Below floor;
amber `#e0b341` for warnings; `#b0def0` for links and in-flight.

Surface tints, all from the mockup: accepted rows `#07100f`, drag-over
highlight `#07171a` (with a `#5fe7e4` border), the "Could not parse" card
`#120a0d` on a `#4a2230` border with a 3px `#ef445f` left edge. Row note
cards are a three-color triple of (border, left edge, text): error
`#4a2230` / `#ef445f` / `#c9a8b0`; warn `#3d3520` / `#e0b341` / `#c9bb95`;
ok `#20343d` / `#5fe7e4` / `#9dc4cc` — 1px border, 2px left edge, 2px
radius, `margin:14px 0 0 42px` so they hang under the row's status dot.

Signature gradients: the fee number uses
`linear-gradient(231.49deg, #61ffe1 10%, #5572f5 62%, #a341ff 100%)`;
the split headline uses the teal range
`linear-gradient(237deg, #61ffe1 0%, #5fe7e4 50%, #5ccbe8 70%, #59a4ee
100%)` on the first line and the violet range
`linear-gradient(237deg, #5433ff 0%, #6e38ff 40%, #853cff 60%, #a341ff
80%, #ac43ff 100%)` on the second, both via `background-clip: text`.
Primary buttons ("Add to queue", "Broadcast all") are teal-on-black
outlines that on hover **fill with the gradient**
(`linear-gradient(231.49deg, #61ffe1 32.13%, #5572f5 69.65%, #a341ff
103.41%)`, `color:#000`, transparent border) — not a color swap; the
disabled state is `#0c0c0c` on `#1f1f1f` with `#4a4a4a` text and
`cursor:not-allowed`. All transitions are `.2s ease-in-out`.

Typography: IBM Plex Sans for prose/UI, IBM Plex Mono for all numbers,
hashes, labels and chips — both bundled locally as woff2 via trunk assets,
no external font CDN. (The mockup's bundle also carries Satoshi and
Poppins from the wider design system; the page uses neither, so they are
not shipped.) Sizes as in the mockup: hero headline 64px desktop /
38px mobile, fee number 124px / 76px (line-height `.86`, letter-spacing
`-4px` / `-2px`), section headings 26px/600, body 13.5px, captions
12.5px, eyebrows 11–13px/500 uppercase with 1.2–1.6px letter-spacing,
mono chips 11px, row name 13px mono, row fee rate 17px/600 mono.

Distinctive card shape: asymmetric border radius — `44px 2px 44px 2px` on
the fee card, `44px 2px 0 0` on the queue stat strip, `0 0 44px 2px` on
the queue table directly beneath it (the two read as one shape, so the
strip carries `border-bottom:0`), `22px 2px 22px 2px` on the FAQ card;
buttons, chips, inputs and note cards are near-square (`2px`). The fee
card alone carries `box-shadow: 2px 4px 10px 0 rgba(0,0,0,.15)`. Small
uppercase letter-spaced eyebrows for section labels; a pulsing dot
animation for in-flight rows (`@keyframes wspulse`, opacity `.35`→`1`,
1s ease-in-out infinite). The page is padded `0 6%` with content capped at
`max-width:1280px`. The single responsive breakpoint is
`(max-width: 860px)`, matched with `window.matchMedia` and held in state —
every layout below has exactly two forms, desktop and mobile.

### Page structure (top to bottom)

Disclosure strip: a sticky bar at the very top (`position:sticky; top:0;
z-index:20`, `rgba(0,0,0,.94)` over a `#1a1a1a` bottom hairline, 42px
tall), holding the mono eyebrow `SECURITY DISCLOSURE`, a 1px×14px `#2a2a2a`
divider, and the link "Coldcard RNG vulnerability, and why you might need
this tool" (underlined in `#4a6470` at 3px offset) followed by a 12px
arrow-out-of-box glyph. It stays visible as the page scrolls.

Context banner: directly below, a bordered card (`#0c0c0c`, `#2a2a2a`
border, 3px `#5fe7e4` left edge, `18px 26px` padding) explaining why the
tool exists, capped at `96ch`: "This tool exists because of a specific
failure: keys generated with predictable randomness can be recovered by
anyone who notices. If that is your situation, moving the coins is a race,
and the public mempool is where you lose it." — closing with a semibold
"Read the disclosure ›" link to the Wizardsardine Coldcard-RNG post.

Hero, two columns
(`grid-template-columns: minmax(0,1.05fr) minmax(0,.95fr)`, 60px gap,
`align-items:center`, `padding:20px 0 60px` over a `#1a1a1a` bottom
hairline; on mobile a 32px-gap column stack with `padding:26px 0 44px`):
left, the eyebrow "Send directly to the miner", the gradient headline
"Get your transactions / mined directly." (line one teal-range, line two
violet-range, split by a `<br>`), and three mono chips 9px apart:
`PSBT · base64`, `RAW TX · hex`, `.TXT .PSBT .TXN .TAR.GZ .ZIP`. Right,
the fee card: eyebrow "Minimum accepted fee rate", Slipstream's
`effective_rate` (section 2) as a huge
gradient mono number with `sat/vB` beside it in 22px `#a1a1a1` mono
(baseline-aligned, 14px gap), a `#1f1f1f` hairline rule
(`margin:26px 0 20px`), and — corrected from the mockup, see departure 9
— the sentence "Anything below this rate will not be mined." The number
comes from `/fee`, polled with
`gloo-timers` every 30 s; while stale (per the API's flag) it dims and
shows a staleness note rather than displaying a dead value as live.

Load transactions: heading plus the mockup's sentence, capped at `70ch` —
"Paste or drop signed PSBTs and raw transactions. We hand each one to MARA
Slipstream, which mines it without ever touching the public mempool." A
live `DETECTED · PSBT` /
`DETECTED · RAW TRANSACTION` / `UNRECOGNISED FORMAT` mono label sits right
of the heading (11px, `#6a6a6a`, baseline-aligned with the heading),
updating as the user types. The `<textarea>` — 214px tall,
`resize:vertical`, 13px mono at `line-height:1.6`, `padding:18px 20px`,
`word-break:break-all`, `#e6e6e6` on `#060606` inside a `#2a2a2a` border,
spellcheck off — shows a placeholder with sample base64 and hex openings
and "or drop a file here". It is itself a drop target, swapping to a
**solid** `#5fe7e4` border over `#07171a` while a file is dragged over it,
as is the page at large; window-level `dragover`/`drop` handlers call
`preventDefault` so a missed drop never navigates the tab away. Controls
row: primary "Add to queue" button (disabled until the paste box parses;
Ctrl/Cmd+Enter queues), a bordered "Choose files" button with an upload
glyph opening a hidden `<input type="file" multiple>` (accept list:
`.txt,.psbt,.txn,.hex,.raw,.tar,.gz,.tgz,.zip`), the caption "Files,
folders or archives: .txt .psbt .txn .tar .tar.gz .zip", and — once items
exist — a quiet "Clear queue" link. Parse failures of a single pasted
entry render a red-left-bordered "Could not parse" card with the reason in
mono. Multi-line pastes split into one queue item per non-blank line.
Unfinalizable PSBTs trigger the finalization modal (section 1): a centered
card in the page's card style (`#0c0c0c`, `#2a2a2a` border, red `#ef445f`
accent) over a dimmed backdrop, titled "PSBT cannot be finalized", listing
the refused item(s) with incomplete-input counts in mono, a one-line
explanation ("Sign all inputs, then load it again."), and a single Close
action; the rest of the load proceeds into the queue behind it.

File expansion happens in the browser, in `unpack.rs`: each dropped or
chosen file is read via `gloo-file`, sniffed by magic bytes (zip
`PK\x03\x04`, gzip `\x1f\x8b`, tar checksum at offset 148 — never by
extension), archives extracted with the pure-Rust crates (`zip`, `tar`,
`flate2`/miniz_oxide) under self-protection limits (256 MiB streamed
decompression budget, 1,000-entry cap, no nested archives — a hostile
archive can at worst disrupt the tab of the person who dropped it; the
server never sees it). Entries sort lexicographically; text files split
per line; binary PSBTs/transactions convert to base64/hex. Every item is
immediately analyzed with `tx-core` in wasm and enters the queue labeled
by origin (`extracted from recovery-batch.tar.gz`, `dropped file`,
`pasted`). Items over the payload cap are flagged at once.

Queue summary: a four-cell stat strip — "In queue" (total, default text),
"Clear the floor" (teal `#61ffe1`), "Below floor" (red `#ef445f`), "Fee
unknown" (muted `#a1a1a1`) — each an 11px uppercase eyebrow over a
28px/600 mono count, `padding:20px 26px`, separated by `#1f1f1f` right
borders (dropped on the last cell of each row). The strip is a grid of
`repeat(4,minmax(0,1fr)) minmax(190px,1.4fr)`, the fifth cell holding the
primary broadcast button flush right; on mobile it becomes
`repeat(2,minmax(0,1fr))` with hairlines under the first two cells and the
button spanning `1 / -1` full-width above a `#1f1f1f` top border. The
button (called "Broadcast all" throughout this document) renders a dynamic
label: `Broadcasting…` while a run is in flight, `Broadcast N
transactions` when more than one item is submittable, otherwise
`Broadcast`. It is disabled only while a run is in flight or nothing is
submittable — where submittable means every queued row that decoded
successfully and has not already been accepted, regardless of its fee
rate. The three fee counts are read-outs, not preconditions; the strip
reports what is in the queue, it does not describe a work list to clear
before the button will fire.

Queue table: header row (desktop only, `#060606` on a `#1f1f1f` hairline,
10.5px uppercase `#6a6a6a`) reading `— / Source / Format / vsize /
Fee rate / Status / —`. Header and rows share one grid,
`26px minmax(0,2.4fr) 110px 130px 150px 120px 30px` with a 16px gap and
`padding:18px 26px` per row over a `#1a1a1a` bottom hairline; on mobile
the header is hidden and the row becomes `flex-wrap` with `10px 12px` gaps
and the name cell at `flex:1 1 60%`. Each row: a 9px round status dot
(teal `#61ffe1` ready/accepted, red `#ef445f` invalid/below-floor, grey
`#4a4a4a` unknown, light-blue `#b0def0` pulsing via `wspulse` while
sending); the item name in 13px mono, ellipsized, with its origin beneath
in 11.5px `#6a6a6a`; a bordered mono format chip (`PSBT` / `RAW TX`,
`justify-self:start`); vsize right-aligned (`n vB`, space-thousands, `n/a`
when unknown); the fee rate right-aligned in 17px/600 mono — `#4a4a4a`
when unknown, `#f4f4f4` at or above the floor, `#ef445f` below — fixed to
two decimals, with a 10.5px `#6a6a6a` sub-line carrying the absolute fee
(`12 345 sats`, space-thousands) or the bare unit `sat/vB` when no fee is
derivable; a colored status label — `Ready`, `Below floor`, `Fee unknown`,
`Invalid`, `Sending…`, `Accepted`, plus `Rejected`, `Rate limited` and
`Failed` from real submission outcomes; and an `×` remove control (17px
`#4a4a4a`, `#ef445f` on hover). `Rejected` and `Failed` rows keep their
place in the queue, carry the reason in an error note card, and gain a
quiet `Retry` control beside the status; nothing but the user's `×` or
"Clear queue" ever removes a row.

Rows carry contextual note cards (the ok/warn/error triples in the tokens
above) for analysis results: "Missing UTXO data for n input(s). Fee and
final scripts could not be validated.", "Finalized locally: n input(s) to m output(s), ready to
extract.", or "Malformed: <reason>" / "Not valid base64 PSBT or hex
transaction data." (Not-fully-signed PSBTs never become rows — they are
refused at load time by the finalization modal, section 1.)

Any row in `Fee unknown` state additionally exposes an inline "total input
value" field: a dashed-border `#060606` panel holding the explanation and
a mono input, from which fee and rate derive live, relabeling the row
Ready or Below floor. The field changes what the row *reports*, never
whether it can be submitted — an untouched Fee-unknown row broadcasts just
the same. This covers raw transactions *and* PSBTs missing
UTXO data for some inputs — the output sum is derivable from a PSBT's
unsigned transaction just as it is from a raw one, so both get the field.
(The mockup gates this panel on an `outSum` field its PSBT branch never
populates, which leaves a partial-UTXO PSBT stranded as an unfixable
`Fee unknown` row; that is a bug in the prop, not a design intent, and the
implementation sets the output sum for both formats. The panel's copy is
correspondingly generalized from the mockup's raw-transaction-only
wording: "This transaction's input amounts are not known, so the fee
cannot be derived locally. Enter the total value being spent to check it
before submitting.")

Every row carries a 12px mono txid line from the moment it is queued — a
`#6a6a6a` `TXID` label followed by the txid, `word-break:break-all` and
freely selectable, with a copy control beside it. `tx-core` computes the
txid from the finalized transaction locally, so it is available whether or
not the transaction is ever successfully broadcast; a user whose
submission bounced must never have to reconstruct the identifier by hand.
The txid renders `#6a6a6a` while the item is unsent, rejected or failed,
and `#61ffe1` once accepted. Accepted rows additionally tint `#07100f` and
link the txid to a block explorer (base URL compiled in) — a link the
other states omit, since an unbroadcast transaction is not there to look
up.

Broadcast behavior: "Broadcast all" iterates every submittable item —
each row that decoded successfully and has not already been accepted,
whatever its fee rate — strictly sequentially, in queue order. Each item's
finalized transaction hex, extracted locally by `tx-core` at queue time,
is sent as one `POST /broadcast`; await the response, then the next —
preserving dependency order end-to-end. Below-floor and Fee-unknown rows
are submitted like any other; the fee labels informed the user, and the
user pressed the button (section 1). The only rows skipped are `Invalid`
ones, which have no transaction to send, and already-`Accepted` ones,
which must not be sent twice.

Outcomes never remove a row. A Slipstream rejection marks it `Rejected`
with the verbatim reason and the loop continues (an independent queue
shouldn't be stranded, and a dependent descendant of a rejected parent
fails on its own merits). A 429 pauses the loop with a visible countdown
from `retry_after_secs` and resumes automatically. A network failure
marks the row `Failed` and pauses the run with a Retry control, never
silently skipping. Every one of those rows keeps its txid, its analysis
and its place in the queue, so pressing "Broadcast all" again re-attempts
exactly the rows that did not succeed — after the user has fixed what
they wanted to fix (added an input value, waited out a floor change) or
simply because the failure was transient. Per-row `Retry` re-attempts a
single item without touching the rest.

FAQ: under the eyebrow "Questions you should ask before using this", an
accordion (single-open, chevron rotating 180° over `.2s`, teal when open)
in a `22px 2px 22px 2px` card, entries divided by `#1a1a1a` hairlines
except the last. The five questions, verbatim from the mockup:

1. "Does this keep my transaction out of the public mempool?"
2. "Does anything I paste get sent to your server?"
3. "What does MARA learn about me?"
4. "Am I guaranteed to get confirmed?"
5. "Why is there a minimum fee rate at all?" — its answer interpolates the
   live floor ("currently N sat/vB").

Answers 1 and 4 are taken verbatim. Answer 5 is verbatim but for its
closing clause: the mockup says a transaction below the floor "needs to be
rebuilt at a higher rate before it can be submitted", which this UI no
longer makes true — it can be submitted, it will simply be rejected. Read
"…before it can be accepted." Answers 2 and 3 must be rewritten,
because the mockup was drawn against a serverless design and its copy
("We operate no server and store nothing"; MARA learns "the IP address it
arrived from") is simply false of the built service. The honest copy is
that parsing, finalization and fee math run in the browser, and pressing
Broadcast sends the finalized hex through this site's own thin relay —
which holds the Slipstream credential, enforces rate limits, stores
nothing, and logs txids only — and that MARA therefore sees the relay's
IP, not the user's (the relay operator sees the user's IP, as any website
does).

Footer: separated by a `#1a1a1a` top rule, "Built by
[Wizardsardine](https://wizardsardine.com)" on the left and the
"Disclosure" and "Slipstream terms" links on the right, 13px, wrapping on
narrow screens. Links are `#b0def0`, hovering to `#5fe7e4`, undecorated.

All authoritative validation is Slipstream's, with the server checking only
the transaction encoding and presence of inputs and outputs; the browser's
analysis exists for instant feedback and its archive limits for tab
self-protection. Neither browser nor server gates a submission on its fee
rate (section 1). `unpack.rs` is pure (bytes in, labeled
text items out) and carries the archive and expansion test suite under
`wasm-bindgen-test` with tar/tgz/zip fixtures, including entry-count and
decompression-budget refusals (no crafted zip-bomb fixture is built — the
budget is exercised with a plainly oversized entry instead, since the
threat here is self-inflicted: a hostile archive can only disrupt the tab
of the person who dropped it); `tx-core`'s analysis tests already run on wasm too
(section 4).

### Deliberate departures from the mockup

The mockup wins on everything except the following nine points — places
where it asserts something untrue of the built service, simulates behavior
it does not implement, or encodes a queue policy that has since been
reversed. Anything not on this list is a mockup detail to be reproduced,
not a decision to be revisited.

1. **FAQ answers 2 and 3** are rewritten. The mockup says "We operate no
   server and store nothing" and that MARA learns "the IP address it
   arrived from"; both are false once submissions pass through a relay
   holding the client code. Corrected copy is specified above.
2. **Unfinalizable PSBTs are refused at load**, via the modal in
   section 1, rather than queued as a warn-note row. The mockup queues
   them with "Not fully signed: n of m inputs finalized" and would then
   submit them like anything else — doubly so now that nothing filters the
   broadcast run (point 6). The modal exists to close that hole; it is the
   one place this design does refuse on the user's behalf, and it does so
   because an unsigned transaction cannot succeed under any fee, not
   because we disagree with the user's judgment.
3. **The broadcast loop is strictly sequential** — one `POST`, awaited,
   then the next. The mockup fans out staggered `setTimeout`s 260 ms apart
   with overlapping lifetimes and invents a random txid; that cannot
   preserve the dependency ordering guaranteed in section 1, nor pause on
   a 429, nor surface a real rejection.
4. **The `· est.` fee suffix is not implemented.** The mockup marks every
   PSBT estimated because it approximates witness size at 108 B/input.
   `tx-core` finalizes and extracts before queueing, so every queued
   item's vsize is exact and the marker would never render.
5. **The paste-box button reads "Add to queue".** The mockup labels it
   `Broadcast`, which collides with the actual broadcast button beneath
   the stat strip — visibly a leftover from an earlier iteration.
6. **Nothing is filtered out of a broadcast run on fee grounds.** The
   mockup's `onBroadcastAll` submits only items whose derived rate clears
   the floor (`e.rate !== null && e.rate >= min`); this design submits
   every valid, not-yet-accepted row and lets Slipstream decide
   (section 1). The fee analysis is displayed, not enforced.
7. **Every row shows its txid from the moment it is queued**, not only
   after a successful broadcast as in the mockup (`hasTxid`, populated
   from a random hex string once its fake submission "succeeds"). The txid
   is derived locally from the finalized transaction, and a rejected or
   failed row keeps it — the user must always be able to walk away with
   the identifier of a transaction they built here.
8. **The fourth stat cell is labeled "Fee unknown", not "Needs input
   value".** With gating removed, nothing *needs* an input value; the
   field is there to inform. The mockup's label would now describe a
   requirement that does not exist.
9. **The fee card reads "Anything below this rate will not be mined",**
   not the mockup's "rejected, not queued". The hero number is
   `effective_rate` (4.0), but the true rejection line is
   `submit_fee_rate` (2.0) — between them a transaction is accepted and
   then quietly never mined (section 2). "Rejected, not queued" would be
   false for exactly the band where a user is most likely to lose coins
   while believing they succeeded.

Beyond these, the mockup's logic is prop scaffolding wherever it stands in
for work `tx-core` and `unpack.rs` do for real: its hand-rolled JS
transaction/PSBT readers, its regex `classify()` heuristics, its
`onExamples` demo seeder, and its `ingest()` path that fakes archive
extraction by synthesizing two to five sample entries from the file's name
length. None of that ships; only the interface it produces does.

## 6. Deployment

All deployment is driven by three bash scripts sharing the same skeleton:
`set -euo pipefail`, colored `log_info`/`log_warn`/`log_error` helpers and a
`die` function, and project-root resolution from `BASH_SOURCE` so each
script works whether invoked from the repo root or from `deploy/`. Each
script supports two modes: with no argument it operates on the local
machine; with a `user@host` argument it rsyncs the project to
`/opt/outofband/src` on the remote after excluding build output, git data,
tool state, and `dev-config.toml`. It first creates the directory over SSH,
then re-executes itself there without
arguments. Nothing requires being run as root; `sudo` is invoked internally
for privileged steps only.

### install.sh — full bootstrap

Idempotent bootstrap of a fresh Debian/Ubuntu server, in order:

1. `apt-get install` prerequisites: `build-essential`, `pkg-config`,
   `curl`, `rsync`, `nginx`, `certbot`,
   `python3-certbot-nginx`.
2. Toolchain if missing: rustup (stable), the `wasm32-unknown-unknown`
   target, and `trunk` (via `cargo install`).
3. System identity and directories: system user `outofband` (no login
   shell), `/etc/outofband`, `/opt/outofband`, `/var/www/outofband`.
4. Build: `cargo build --release -p broadcast-api`; `trunk build --release`
   in `crates/broadcast-frontend`.
5. Install artifacts: binary to `/usr/local/bin/broadcast-api`; trunk
   `dist/` contents to `/var/www/outofband/`; `deploy/config.toml` to
   `/etc/outofband/config.toml` **only if absent** — an existing (edited)
   config is never overwritten — then `chown root:outofband` and `chmod 640`.
6. systemd: install the unit, `daemon-reload`, enable it, and restart it so
   repeated installs cannot leave an old process running.
7. nginx: install the shared application and security snippets, install the
   port-80 server wrapper when absent, symlink it into `sites-enabled/`,
   remove the default site, run `nginx -t`, and reload.
8. Health check: curl `http://127.0.0.1/health` through nginx and fail
   loudly if it doesn't answer.
9. Print the remaining manual steps: set the real `server_name`, fill
   `client_code` and the two endpoint paths in `/etc/outofband/config.toml`,
   restart the service, run certbot.

TLS: the nginx config ships as a plain port-80 server so the first install
works before DNS or certificates exist. `install.sh` accepts an optional
`--domain example.com --email you@example.com`; when given and the domain
already resolves to the host, it sets `server_name` and runs
`certbot --nginx -d <domain> --redirect --agree-tos -m <email> -n`, which
rewrites the site for 443 with the Let's Encrypt certificate and installs
the systemd renewal timer. Without the flag, the README documents the same
one-liner to run manually after DNS is ready.

### update.sh — soft redeploy

Code-only: rsync source (remote mode), rebuild binary and WASM, reinstall
binary, dist, systemd unit, and managed nginx snippets, then restart the
service and validate and reload nginx. It never touches users, the live
config in `/etc/outofband`, or certificates. A legacy Certbot site without
the managed snippet emits a migration warning rather than replacing TLS.

### clean.sh — teardown

Removes everything install.sh created: stops and disables the service,
deletes the unit, the nginx site (available + enabled symlink), the binary,
`/var/www/outofband`, `/opt/outofband`, and — after an explicit confirmation
prompt, since it holds the client code — `/etc/outofband`. Leaves apt
packages and the rust toolchain in place.

### nginx configuration

`deploy/nginx/outofband.conf` is a small port-80 server wrapper. It includes
`/etc/nginx/snippets/outofband-app.conf`, generated from
`deploy/nginx/outofband-app.conf`. The application snippet owns routes,
headers, cache policy, logs, the payload cap, and request limiting. Keeping
that policy outside the server wrapper lets updates replace it without
overwriting the TLS directives Certbot adds to the wrapper.

The API proxy overwrites both client-IP headers with `$remote_addr`.
Application code trusts them only from a localhost peer. nginx returns 429
when its coarse limiter fires; the frontend retries both nginx's non-JSON
response and the application's JSON 429 response. Security headers are
included again in nested cache-policy locations because an nginx
`add_header` in a child location replaces inherited headers.

### systemd unit — `deploy/systemd/broadcast-api.service`

```ini
[Unit]
Description=Outofband broadcast API (MARA Slipstream relay)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=outofband
Group=outofband
WorkingDirectory=/opt/outofband
ExecStart=/usr/local/bin/broadcast-api /etc/outofband/config.toml
Restart=always
RestartSec=10s
NoNewPrivileges=true
PrivateTmp=true
PrivateDevices=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
ProtectClock=true
ProtectHostname=true
LockPersonality=true
RestrictRealtime=true
RestrictSUIDSGID=true
CapabilityBoundingSet=
AmbientCapabilities=
ReadOnlyPaths=/etc/outofband

[Install]
WantedBy=multi-user.target
```

`network-online.target` because the service is useless without outbound
HTTPS to MARA; `ProtectSystem=strict` is viable because the service writes
nothing to disk.

### Developer commands — `.justfile`

```just
clippy:
    cargo clippy
    cd crates/broadcast-frontend && cargo clippy --target wasm32-unknown-unknown

fmt:
    cargo fmt
    cd crates/broadcast-frontend && cargo fmt

test:
    cargo test
    cd crates/broadcast-frontend && wasm-pack test --headless --firefox

# Run the whole stack locally over plain HTTP: no nginx, no TLS, no
# systemd, no root. Backend on 127.0.0.1:3010, frontend on :8080.
run:
    #!/usr/bin/env bash
    set -euo pipefail
    [ -f dev-config.toml ] || { install -m 600 deploy/config.toml dev-config.toml; \
        echo "created dev-config.toml; fill client_code to broadcast"; }
    chmod 600 dev-config.toml
    cargo run -p broadcast-api -- dev-config.toml &
    trap 'kill %1 2>/dev/null || true' EXIT
    cd crates/broadcast-frontend && trunk serve --open

serve:
    cd crates/broadcast-frontend && trunk serve   # frontend only, :3010 must be up

deploy remote:
    ./deploy/install.sh {{ quote(remote) }}

local:
    ./deploy/install.sh

update:
    ./deploy/update.sh

update-remote remote:
    ./deploy/update.sh {{ quote(remote) }}

clean-local:
    ./deploy/clean.sh

clean-remote remote:
    ./deploy/clean.sh {{ quote(remote) }}
```

`just run` is the everyday development command and the only one that
exercises the whole product without touching the system: it starts
`broadcast-api` on `127.0.0.1:3010` and `trunk serve` on `:8080` in front
of it, over plain HTTP, and kills the backend when the frontend exits.
Nothing is installed, no port below 1024 is bound, no root is needed, and
TLS — which exists only in the nginx layer that certbot rewrites
(section 6) — is simply absent. The deploy scripts are therefore never in
the loop while developing; `just local` remains available for rehearsing
a real install on this machine, and it is the only local command that
wants `sudo`.

The config it uses is `dev-config.toml` at the repo root, copied with mode
`600` from `deploy/config.toml` on first run and listed in `.gitignore`
because it may hold a real client code. Remote deployment also excludes it
explicitly. The page is fully usable before that code is
filled in: the fee card polls `GET /api/rates`, which needs no credential
(section 2), so the hero renders a live floor and the queue analyzes
transactions normally — only pressing Broadcast fails, with Slipstream's
own "Client codes are currently required" message surfaced per row, which
is exactly the behavior a rejection should produce.

Because the routes sit at the root rather than under one prefix,
`Trunk.toml` carries three `[[proxy]]` entries — `/fee`, `/broadcast`,
`/health`, each to `http://127.0.0.1:3010` — instead of a single prefix
proxy, mirroring the three-path nginx location above. Those proxies are
what make `just run` work without nginx.

The native commands (`cargo clippy/test/fmt` at the root) cover the backend
and client crates; the frontend is exercised separately against the wasm
target — clippy via cargo, tests via `wasm-bindgen-test` in a headless
browser, since `unpack.rs` exercises real extraction in the wasm
environment.

## 7. Security and correctness notes

The service is an open relay by design, so the protections are: the
per-IP sliding-window transaction budget, nginx `limit_req` shedding raw
floods in front of it, the 1 MiB (configurable) payload cap enforced by
both nginx and axum, strict input parsing (anything that doesn't decode as
a well-formed PSBT or transaction is rejected before any outbound call),
and no logging of transaction payloads at info level — txids only, since
payloads may correspond to not-yet-broadcast transactions the user
considers private.

The server's input surface is deliberately tiny: one JSON string per
request that must hex-decode to a well-formed transaction with at least one
input and output. Script and chain-context validation remains Slipstream's
job. The bounded server surface has no PSBT parsing, multipart, file uploads,
or decompression, so the
decompression-amplification DoS class does not exist here. Worst-case
memory per in-flight request is the payload cap; no concurrency semaphore
is needed. Archive parsing happens only in the browser, where a hostile
file affects only the tab of the user who dropped it, and even there
extraction streams against a fixed budget, caps entry counts, refuses
nested archives, and never touches a filesystem — tar path-traversal names
are inert labels, sanitized before display. No system C library is linked
anywhere — `reqwest` uses `rustls`, not OpenSSL — though the dependency
graph does vendor C in two places, libsecp256k1 and the rustls crypto
provider, both compiled from source (section 3).

The deployed Slipstream client code exists only in
`/etc/outofband/config.toml` (owned root:outofband, mode 640). The local
development config has mode 600 and is excluded from git and every remote
rsync. The client crate redacts the configured value from every surfaced
upstream error or message.
Because whoever holds the code can submit under this account — and MARA
offers no recourse for misuse — the code should be rotated (via
foundation@mara.com) if it has ever been pasted into email threads, chats,
issue trackers, or shell history; the deployed service is the only place it
should live. The backend binds to localhost only; the sole public surface
is nginx on 80/443. UFW guidance in the README: allow 22, 80, 443, nothing
else.

Honest limitations to document in the README rather than hide: nothing in
this service validates a fee — not the server, not the browser. The
displayed rate is analysis, Slipstream is the only judge, and a
below-floor transaction is submitted and bounced rather than withheld, so
both the web UI and a scripted caller can send any locally accepted
transaction encoding and receive MARA's validation result. That rejection consumes a slot in the
per-IP rate-limit budget like any other submission, which is the practical
cost of not gating. The displayed minimum fee is additionally a cached
value that can lag MARA's real threshold by up to the poll interval — a
transaction whose rate looked comfortable can still bounce inside that
window, and the error message says so. A rejection cannot be classified
either: Slipstream answers with prose, so "fee too low" and "consensus
failure" reach the user as MARA's own sentence rather than a typed error.
CPFP is unsupported — a low-fee parent is judged alone and rejected
(section 2). Queue
behavior (submitting every valid row, keeping failed rows with their
txids, sequential ordering) is frontend policy, not a server guarantee,
since the server sees one transaction at a time. And the default 1 MiB
payload cap excludes the very largest non-standard transactions (~8 MiB
hex) unless raised in config.

## 8. Milestones

Task 0 (transcribing the endpoint URLs, response shapes and error codes
from the Slipstream API documentation into `config.toml` and the
`slipstream-client` fixtures) is **complete** — section 2 holds the
verified values, confirmed against the live API on 2026-08-02. The
milestones below are the narrative shape of the work; the build itself
was split into smaller reviewable changesets. Task 1 builds `slipstream-client` and `tx-core` with
native unit tests plus a wasm build and browser integration tests (Slipstream wire format, PSBT
finalization matrix, vsize/fee computation, fee-rate comparison against a
floor, txid derivation from a finalized transaction), plus the frontend's
`unpack.rs` under `wasm-bindgen-test` (tar/tgz/zip fixtures, text-file
expansion, ordering, the extraction limits).
Task 2 assembles `broadcast-api` (routes, fee cache, sliding-window rate
limiter, payload cap) and exercises it end-to-end with a mocked Slipstream
server, including a sequential dependent two-transaction submission that
must arrive in order and a rate-limit exhaustion-and-recovery scenario.
Task 3 is the Yew frontend against the local API, implemented to the
section-5 specification and verified against the mockup: design tokens and
layout, the queue with live `tx-core` analysis and a locally derived txid
on every row, the total-input-value field, drag-and-drop with in-browser
extraction, and the sequential broadcast loop with its 429-pause and
failure-pause behavior — including a run where a below-floor item is
submitted, rejected, retained with its txid and reason, and successfully
retried. Task 4 is
the deploy tooling, tested on a
throwaway Debian VM from `install.sh` through certbot to a successful
broadcast, then `update.sh` and `clean.sh`. Task 5 is documentation:
README (deploy, full config key reference including when to raise
`max_payload_bytes`, certbot, `journalctl -u broadcast-api` / log
locations, firewall, update/teardown, and the API note for scripted users:
one finalized transaction hex per request — finalize PSBTs locally first).
