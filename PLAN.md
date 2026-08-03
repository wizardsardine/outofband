# Outofband — Plan

Single-page web service to broadcast Bitcoin transactions through the MARA
Slipstream direct-submission service. The whole product is a static page:
parsing, PSBT finalization, fee maths and the submission itself all run in
the browser, and the submission goes straight from the tab to
`slipstream.mara.com`. Rust throughout, Yew/WASM, no hand-written
JavaScript, with batch submission via drag-and-drop (single files, several
files, or tar/tar.gz/zip archives unpacked in the browser), and one-command
deployment on a fresh Debian server behind nginx with Let's Encrypt TLS.

This document is self-contained: it specifies the architecture, every
convention, the wire formats, and the deployment tooling in enough detail
to implement the project from this file alone.

## Summary

The rest of this document (sections 1 to 7) is the implementation
specification, reference material to work from rather than to read
through. This section is the whole design.

### What it is

A one-page site that hands Bitcoin transactions to MARA Slipstream, which
mines them directly instead of gossiping them through the public mempool.
It exists for the case where a key is known to be compromised (predictable
RNG), and a normal broadcast would lose the race to whoever else can spend
those coins.

There is no backend. A deployment is nginx serving a wasm bundle, so
nothing pasted into the page ever reaches this project's host, and the only
host that ever receives a transaction is MARA's. The dependency graph does
still vendor C in one place, libsecp256k1, which compiles to wasm along
with everything else (section 3).

### Flow

```
browser
  paste box │ dropped files │ dropped .tar/.tar.gz/.zip
      └─> unpack.rs   archives extracted in-browser, magic-byte sniffed,
      │               entries sorted lexicographically
      └─> tx-core     detect → parse → analyze → finalize → extract hex
                      → vsize, fee rate (when derivable), txid
      └─> queue row   one per transaction, nothing sent yet

  "Send batch"     sequential, one request per tx, in queue order
      │
      ▼
MARA Slipstream, called from the tab
  POST /api/transactions {tx_hex}
      → {status, message}: accepted, rejected, or rate limited
        (the fee floor is theirs alone)
```

### The four decisions everything else follows from

1. **The browser talks to Slipstream directly.** Both endpoints this
   project uses are uncredentialed and serve permissive CORS (section 2),
   so a relay would be a hop that holds no secret, sees every transaction
   it forwards, and adds its own availability to MARA's. Without one there
   is no server input surface to defend, no rate limiter to run, no config
   file to protect, and nothing pasted into the page reaches this
   project's host. Costs, accepted: MARA sees the user's IP address rather
   than a relay's, and MARA can break the page with no deploy on our side
   by tightening CORS or moving the endpoint.
2. **All analysis happens in the browser.** `tx-core` compiles to wasm
   and does format detection, PSBT analysis, finalization, extraction,
   vsize, fee and txid. Slipstream only ever sees finalized hex. Queueing
   500 items causes no network traffic at all; nothing is checked against
   any host until a send.
3. **Fee rates are displayed, never enforced.** Nothing in this page
   refuses a submission over its fee. A below-floor transaction is
   submitted and bounced, because that costs one round trip and
   Slipstream's answer is more authoritative than a comparison against a
   floor polled up to 30 seconds ago. MARA's no-recourse warning is about
   transactions that *succeed*; withholding never protected against those.
4. **Nothing leaves the queue.** Rejected, rate-limited and failed rows
   keep their place, their analysis, their reason and their txid, and can
   be retried individually or by pressing Send batch again. Every row
   carries a locally derived txid from the moment it is queued, so a user
   whose submission bounced can still identify the transaction they
   built. Only `×` or "Clear queue" removes a row.

### Crates

| crate                | kind             | role                               |
|----------------------|------------------|------------------------------------|
| `tx-core`            | lib, native+wasm | decode, finalize, vsize, fee, txid |
| `broadcast-frontend` | bin (Yew, trunk) | page, queue, unpack, submit loop   |

`tx-core` building for `wasm32-unknown-unknown` is a hard requirement, not
an aspiration: everything the page reports and everything it submits comes
out of that crate, and its native test suite is only meaningful because
the wasm build is the same code.

### The two calls

| Method | URL                 | Body / response                         |
|--------|---------------------|-----------------------------------------|
| GET    | `/api/rates`        | `effective_rate`, the floor in the hero |
| POST   | `/api/transactions` | `{tx_hex}` → `{status, message}`        |

Both against `https://slipstream.mara.com`, overridable at build time with
`SLIPSTREAM_BASE_URL`, both uncredentialed, both CORS-permissive. Rates
are polled every 30 s; a failed poll keeps the last value and marks it
stale rather than showing a dead number as live. `{status, message}` comes
back on 200 and 400 alike, so the outcome is read from the body and not
the status line: `success` accepts, `error` rejects with MARA's own
sentence, 429 is a rate limit, anything else is a failure. `Retry-After`
is not CORS-safelisted and so is usually unreadable from a browser, which
is why a rate-limited row falls back to waiting 5, then 10, then 20
seconds before giving up.

### Frontend

- Input: paste (one entry per non-blank line, `#` comments skipped),
  files, or `.tar`/`.tar.gz`/`.tgz`/`.zip` archives, all sniffed by
  content and never by extension.
- Archive entries and files sort lexicographically and submit strictly
  sequentially in that order, so naming files `01-parent`, `02-child`
  broadcasts a dependent chain correctly. This ordering is guaranteed.
- Extraction is in-browser under self-protection limits (256 MiB budget,
  1,000 entries, no nesting). A hostile archive can at worst disrupt the
  tab of the person who dropped it; it goes nowhere else.
- A PSBT that cannot be finalized never enters the queue: one modal per
  load lists every refused item. It is the only thing the UI refuses on
  the user's behalf, and only because an unsigned transaction cannot
  succeed under any fee.
- Each row: format chip, vsize, txid, fee rate against the live floor,
  status, a colored note card, and remove/retry controls. Fee-unknown
  rows (raw transactions, and PSBTs missing UTXO data) expose a
  "total input value" field that derives the rate, informational only.
- Sending: sequential, awaited. A 429 counts down and retries the same
  row in place; a rejection marks the row and continues; a network
  failure, or a 429 that outlasts the retries, marks the row and pauses
  the run.
- `UI_MOCKUP.html` at the repo root is the authoritative visual
  reference. Section 4 transcribes it and lists the eight deliberate
  departures; anything not on that list is a detail to reproduce.

### Deployment

There is no configuration file and no secret to keep anywhere: the page is
built once and served as static files. `SLIPSTREAM_BASE_URL` at build time
is the only knob, and it exists for pointing a test deployment at another
host.

Three bash scripts under `deploy/`, each running locally with no argument
or against `user@host` by rsync and re-exec: `install.sh` (apt deps,
toolchain, build, nginx site and snippets, health check, optional
`--domain/--email` certbot), `update.sh` (rebuild the bundle, reinstall it
and the managed nginx snippets, never touching certificates), `clean.sh`
(removes the site, the snippets and the install directories, and clears
out what an earlier backend deployment left behind).

### Slipstream, in one paragraph

Transcribed from the live API on 2026-08-02 and re-probed on 2026-08-03
(section 2). Fee numbers come from `GET /api/rates`, uncredentialed; the
headline figure is `effective_rate` (4.0 sat/vB at time of writing), the
rate at which a transaction actually gets mined, not `submit_fee_rate`
(2.0), which only buys admission to a mempool that may never mine it.
Submission is `POST /api/transactions {tx_hex}`. A `client_code` is
optional, as the spec always said and as the service now agrees again: it
buys a reduced fee threshold, not admission, and this page sends none. The
response is `{status, message}` with **no txid**, which is why deriving
the txid locally is load-bearing. Both endpoints reflect any `Origin`, on
the preflight and on the real response, which is what lets the page call
them from the tab at all. A packages endpoint exists for CPFP; this
project does not use it, so CPFP is out of scope.

### Accepted limitations

No fee validation anywhere: any well-formed transaction encoding with
inputs and outputs is submitted, and Slipstream performs the actual
validation. The displayed floor is polled and can lag the real threshold
by up to 30 seconds, so a comfortable-looking rate can still bounce.
Rejections are prose, so "fee too low" and "consensus failure" reach the
user as MARA's own sentence rather than a typed error. MARA sees the
user's IP address, because the connection is the user's own and nothing
here proxies it, and MARA can break the page by tightening CORS on
`/api/transactions` or moving the endpoint. CPFP is unsupported. Queue
behavior is this page's own policy and binds nobody else. The 1 MiB cap on
queued transaction hex excludes the largest non-standard transactions
(~8 MiB hex).

## 1. What the service does

The user opens a single dark-themed page (full UI specification in
section 4, matching the approved mockup). A hero shows Slipstream's live
`effective_rate` — "the floor" — refreshed periodically, with the rule
stated plainly: anything below it will not be mined.
Below, the user loads transactions two ways: pasting into a text area
(a single PSBT in base64 or raw transaction in hex — or several, one per
line) and dropping/choosing files — single files, several at once, or
archives (`.tar`, `.tar.gz`/`.tgz`, `.zip`), unpacked in the browser
(section 4). Everything loaded lands in a queue table where each item is
analyzed locally and shows its format, vsize, txid, fee rate when
derivable, and how that rate compares to the floor. "Send batch" then
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
"Send batch" submits every queued transaction, including Below-floor
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

The consequences are recorded honestly: below-floor items spend whatever
allowance Slipstream grants the user's own address on submissions that
predictably bounce, and the queue's counts of Below-floor and Fee-unknown
items are advisory rather than a work list to clear before broadcasting.
(The user-entered input total is unverifiable by us and gates nothing at
all: it exists so the user can see a rate before committing. Slipstream
remains the final validator.)

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
applying rust-bitcoin's fee-rate limit. UTXO transaction IDs, output
indexes, and duplicate witness and non-witness UTXO descriptions must
agree before either analysis or finalization proceeds. What the frontend
holds for every queued item is the finalized transaction hex and its txid;
that hex is the only thing that ever leaves the browser. An
already-finalized PSBT missing UTXO data can be
extracted at the same trust level as a raw transaction, with an explicit
warning that its fee and final scripts were not validated. Other PSBTs that
cannot be finalized never leave the browser. The modal lists every refused
item and its actual finalization reason while the rest of the load queues
normally.
Binary uploads are handled the same way — bytes in, analyzed item out —
so no conversion rules burden the user.

What leaves the page is consequently exactly one thing: finalized raw
transaction hex, one transaction per request, in the `{tx_hex}` body
Slipstream itself defines. No PSBT, no archive, no multipart and no fee
number crosses the wire; the analysis stays in the tab that produced it.
Before a row is submittable its hex must decode as a well-formed
`bitcoin::Transaction` with non-empty inputs and outputs and fit the
payload cap, which is a guard against wasting a round trip rather than
validation: scripts and chain context are Slipstream's to judge.
Fee-floor enforcement exists nowhere in this page. Slipstream is the sole
and final authority on the floor, and the page is a queue and an analyzer
in front of it. Anyone scripting against Slipstream posts the same body
without us, finalizing PSBTs locally first (e.g. with `bitcoin-cli
finalizepsbt`); what the page adds is the analysis, the ordering and the
queue, not access.

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
**verified against the live API on 2026-08-02, then re-probed on
2026-08-03**. Where the spec and the deployed service disagree, the
service wins and the disagreement is noted.

### Endpoints

| Method | Path                         | Cred | Purpose                     |
|--------|------------------------------|------|-----------------------------|
| GET    | `/api/rates`                 | none | the fee numbers (see below) |
| GET    | `/api/system`                | none | version, height, floor      |
| GET    | `/api/block-info`            | none | height and both fee rates   |
| GET    | `/api/transactions/status`   | none | confirmation state by tx_id |
| POST   | `/api/transactions`          | none | submit one transaction      |
| POST   | `/api/transactions/packages` | none | 2 to 25 txs as a package    |
| POST   | `/api/mempool/tests`         | auth | testmempoolaccept           |

This project uses exactly two: `GET /api/rates` and
`POST /api/transactions`.

### Credentials, and why the page needs none

The spec's `security` block declares no authentication at all. For
submission that was wrong on 2026-08-02 and is right again on 2026-08-03.
That reversal is the fact this whole design turns on, so it is recorded in
both directions.

**The client code is optional.** The spec annotates `client_code` as an
optional field "used to apply a reduced fee rate threshold". On
2026-08-02 a live `POST /api/transactions` carrying only `tx_hex`
contradicted that with a `400`:

```json
{"status":"error","message":"Client codes are currently required to submit
transactions. Contact us at foundation@mara.com for more information."}
```

Re-probed on 2026-08-03, the same shape of request answers what the spec
describes: `{"tx_hex":"00"}` returns
`400 {"status":"error","message":"Failed to deserialize transaction"}`,
which is a complaint about the transaction and not about a missing
credential. So the code is a discount, not an entry ticket, and this
project sends none. It could not usefully hold one anyway: a code compiled
into a wasm bundle is a published code.

**An `Authorization` header** gates `POST /api/mempool/tests`, which
returns `401 {"is_success":false,"message":"Missing or invalid
Authorization header"}` without one. That credential is undocumented and
we do not have it, so the dry-run endpoint is unavailable to this project
regardless of design preference.

Everything the page does, on load and on send, is therefore
uncredentialed.

### CORS

Verified live on 2026-08-03 against both endpoints this project uses:
`access-control-allow-origin` reflects whatever `Origin` is sent, on the
preflight and on the real response alike; `POST` is allowed; a
`content-type: application/json` request header is allowed. So a browser
on any origin can call `/api/rates` and `/api/transactions` and read the
answers, which is what makes a page with no backend possible.

One header does not come through: `Retry-After` is not on the CORS
safelist and Slipstream does not name it in
`Access-Control-Expose-Headers`, so the browser hides it even when
Slipstream sends it. Anything reacting to a `429` has to work without it
(section 4), reading it only as a bonus when a proxy happens to expose it.

CORS is MARA's to change, and along with the client-code requirement it is
what can break this page with no deploy on our side. The failure mode is
at least loud: a blocked request is indistinguishable from an unreachable
host in a browser, so every row fails with the same unreachable message
rather than failing quietly.

### The fee numbers

`GET /api/rates` (optionally `?client_code=…`, which applies volume
discounts to the result; the page sends none) returns seven fields. Live
values on 2026-08-02:

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
{"tx_hex": "<raw_transaction_hex_string>"}
```

`client_code` is an optional sibling field (see above) and this page omits
it. `skip_mempool_submission: bool` also exists (validate and store
without relaying); this project never sets it. The response, on both `200`
and `400`, is:

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
failures return `{is_success, message}`. Only the first is read here: a
failed rates poll needs no message, it just keeps the previous number and
marks it stale.

Both paths are constants in the frontend's `slipstream.rs`, and the base
URL is `option_env!("SLIPSTREAM_BASE_URL")` falling back to
`https://slipstream.mara.com`, so a test deployment can be pointed
elsewhere at build time. A path change at MARA is a rebuild, which for a
page whose entire delivery is a rebuild costs nothing.

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
package path would mean a second request shape, a second queue mode, and
an ordering the user has to declare rather than one derived from file
names; it is a documented follow-up, not a first release.

MARA also states plainly that Slipstream operates without technical or
customer support and at the submitter's own risk: an incorrectly built
transaction or an incorrect fee cannot be helped after the fact. That
warning is a design input, not just a disclaimer — it is why every queued
item is analyzed and its exact fee rate shown against the live floor
before the user commits (sections 1 and 4), and why a transaction with no
inputs or no outputs never becomes a submittable row (section 1). What the
warning does *not* justify is refusing
to submit on the user's behalf: the mistakes MARA cannot help with are
mistakes that get mined, and a rejected submission costs a round trip and
nothing else. Analysis informs the decision; it does not make it
(section 1).

## 3. Repository layout and workspace conventions

The project is a single Cargo workspace of two crates, with all deployment
material under `deploy/`:

```
outofband/
├── Cargo.toml                    # workspace: members, deps, lints
├── Cargo.lock                    # committed
├── rust-toolchain.toml           # 1.88, wasm target, clippy, rustfmt
├── .justfile                     # developer commands (see section 5)
├── .gitignore                    # target/, dist/
├── README.md                     # user-facing: deploy, operations
├── UI_MOCKUP.html                # authoritative UI reference (section 4)
├── crates/
│   ├── tx-core/                  # shared lib: detection, PSBT analysis
│   │   ├── src/lib.rs            #   and finalization, vsize + fee math
│   │   └── tests/                #   native tests and their fixtures
│   └── broadcast-frontend/       # Yew CSR app built with trunk (wasm)
│       ├── Trunk.toml
│       ├── index.html            # HTML shell, trunk asset declarations
│       ├── assets/               # style.css and the IBM Plex woff2 files
│       ├── tests/fixtures/       # tar/tgz/zip archives for unpack tests
│       └── src/
│           ├── main.rs           # mounts the app
│           ├── app.rs            # page composition, shared state
│           ├── tokens.rs         # design tokens and page constants
│           ├── queue.rs          # queue item model, analysis, notes
│           ├── slipstream.rs     # the two MARA calls, outcome ladder
│           ├── unpack.rs         # in-browser tar/tgz/zip extraction
│           ├── components/       # one module per section of the page
│           └── hooks/            # fee poll, file load, queue, breakpoint
└── deploy/
    ├── install.sh                # full bootstrap of a fresh Debian server
    ├── update.sh                 # rebuild and reinstall the bundle
    ├── clean.sh                  # remove everything install.sh created
    ├── nginx/outofband.conf      # port-80 server wrapper
    ├── nginx/outofband-app.conf  # managed routes, cache policy, logs
    └── nginx/outofband-security-headers.conf
```

Workspace `Cargo.toml` conventions: `resolver = "3"` (edition 2024's
resolver; it must be set explicitly at the workspace root or cargo warns);
`[workspace.package]` sets `version`, `edition = "2024"`,
`rust-version = "1.88"`, inherited by every crate via
`version.workspace = true` etc.; all dependency versions are declared once
under `[workspace.dependencies]` and referenced from crates with
`foo.workspace = true`; `default-members` lists `tx-core` alone, so a bare
`cargo build`/`cargo test` never tries to build the frontend for the host
target and every workspace-wide command needs `--workspace` to cover it.
Shared lint policy, inherited by every crate through
`[lints] workspace = true`:

```toml
[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
complexity  = { level = "deny", priority = -1 }
perf        = { level = "warn", priority = -1 }
```

`tx-core` depends on `bitcoin 0.32` (`default-features = false`, features
`std` and `base64` for PSBT serialization) and `miniscript 12`
(`default-features = false`, feature `std`, pure Rust, tracking
`bitcoin 0.32`) and on nothing else. `miniscript` is required rather than
optional: `bitcoin` alone can `extract_tx()` an already-finalized PSBT but
cannot *finalize* one, the finalizer lives in `miniscript::psbt::PsbtExt`,
and finalizing a signed-but-unfinalized PSBT is exactly what section 1
promises. The crate compiles for both the host and
`wasm32-unknown-unknown`, which is a hard requirement enforced by the
justfile: the native tests are the only automated proof of the maths, and
they are only worth anything because the wasm build is the same code.

The frontend adds `yew 0.21` (feature `csr`), `wasm-bindgen 0.2`,
`gloo-net 0.6` (the two Slipstream calls), `gloo-timers 0.3` (feature
`futures`, the fee poll and the rate-limit countdown), `gloo-file 0.4`
(feature `futures`, async file reads), `wasm-bindgen-futures 0.4`,
`web-sys 0.3` with the features the page needs (`MediaQueryList`,
`MediaQueryListEvent`, `Window`, `HtmlInputElement`,
`HtmlTextAreaElement`, `KeyboardEvent`, `Navigator`, `Clipboard`,
`DragEvent`, `DataTransfer`, `FileList`, `File`), `serde 1` (derive) and
`serde_json 1` for the wire bodies, `base64 0.22`, `bitcoin`, and, because
archive extraction runs in the browser, the archive crates compiled to
wasm: `tar 0.4`, `flate2 1` (the `rust_backend`/miniz_oxide backend) and
`zip 2` (`default-features = false`, feature `deflate` only). All three
are pure Rust and build cleanly for `wasm32-unknown-unknown`. Browser
tests use `wasm-bindgen-test 0.3`.

On "no C/C++", stated precisely rather than as a slogan, because the
convenient version of this claim is false. **C reaches the browser.**
`bitcoin 0.32` requires `secp256k1 0.29`, which vendors libsecp256k1 (C,
built with `cc`), and it compiles to wasm along with everything else, so
the shipped `.wasm` contains C-derived code. It is vendored and compiled
from source as part of the build, not linked from the system.

What is actually true, and what the README should say: this workspace
contains no C or C++ source of its own; nothing links a *system* C
library; the archive stack (`tar`, `flate2`/miniz_oxide, `zip`) is pure
Rust; and the only toolchain requirement beyond rustc is a working `cc`,
which `build-essential` provides. A literally C-free dependency graph is
not achievable for a project that validates Bitcoin signatures, and
claiming one would be a lie. Supported archive formats are exactly tar,
tar.gz/tgz, and zip. The IBM Plex Sans and IBM Plex Mono woff2 files ship
as trunk assets in the frontend crate, the exact weights the mockup
declares (Sans 200/300/400/400-italic/500/600/700, Mono
300/400/500/600/700), extracted from `UI_MOCKUP.html`'s bundle so the
rendered page matches it byte for byte.

## 4. Frontend: `broadcast-frontend`

Yew 0.21 CSR application built with trunk (`trunk build --release`) into
static `index.html` + wasm-bindgen `.js` glue + `.wasm`, served by nginx
from `/var/www/outofband/`. No hand-written JavaScript; the only JS shipped
is the loader trunk generates. State lives in a handful of hooks; the two
Slipstream calls go through `gloo-net` in `slipstream.rs`; item analysis
calls `tx-core` directly in wasm. This crate is the whole product.

`UI_MOCKUP.html` at the repository root is the **authoritative visual
reference**. Where this section and the mockup disagree on layout, color,
spacing, radius, sizing or copy, the mockup wins and this section is to be
corrected, with the sole exception of the eight departures enumerated at
the end of this section, which exist because the mockup is a static prop:
it simulates behavior it does not implement and encodes a queue policy
that has since been reversed. Everything else below is transcribed from
it.

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
Primary buttons ("Add to queue", "Send batch") are teal-on-black
outlines that on hover **fill with the gradient**
(`linear-gradient(231.49deg, #61ffe1 32.13%, #5572f5 69.65%, #a341ff
103.41%)`, `color:#000`, transparent border) — not a color swap. Pressed
keeps that fill but adds `filter:brightness(.82)` and
`transform:translateY(1px)` with the transition suppressed, so a click
registers instantly instead of easing; the rule must follow the hover rule
to win while the pointer is held. The disabled state is `#0c0c0c` on
`#1f1f1f` with `#4a4a4a` text and `cursor:not-allowed`. All other
transitions are `.2s ease-in-out`. Each queue row's own send control uses
the same three states at row scale (11px, `6px 12px` padding), bordered
rather than bare text so it reads as something to press.

A button whose label changes never resizes. Each one is an `inline-grid`
with a single named area holding two children stacked in the same cell: a
`visibility:hidden` sizer carrying the widest label that button can show,
and the visible label on top. The main button reserves the wider of
`Send batch (n)` and `Sending…` for its current count; a row reserves
`Retry`. Reserving the width matters most at the moment of the click,
when the label changes to `Sending…` and the control would otherwise
shift out from under the pointer.

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
(`margin:26px 0 20px`), and, corrected from the mockup (departure 8), the
sentence "Anything below this rate will not be mined." The number comes
from `GET /api/rates`, polled with `gloo-timers` every 30 s. A failed poll
keeps the last known rate rather than wiping it, and marks it stale: the
card then dims and adds "This rate may be out of date." A rate that is not
a usable positive number counts as a failed poll, since rendering
0 sat/vB as live would put every queued transaction above the floor.

Once the queue holds anything, the stat strip and queue table are rendered
**above** the load section rather than below it, departing from the
mockup's order. The mockup only ever showed an empty-then-filled page read
top to bottom; in use, a user who has already loaded transactions comes
back to act on the queue, and a 214px textarea between the hero and the
rows buries exactly what they returned for. The load section keeps its
place when the queue is empty.

Load transactions: heading plus the mockup's sentence, capped at `70ch` —
"Paste or drop signed PSBTs and raw transactions. Your browser sends each
one straight to MARA Slipstream, which mines it without ever touching the
public mempool." (The mockup's "We hand each one to MARA Slipstream" names
the wrong sender: there is no "we" in the path.) A
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
archive can at worst disrupt the tab of the person who dropped it, since
it goes nowhere else). Entries sort lexicographically; text files split
per line; binary PSBTs/transactions convert to base64/hex. Every item is
immediately analyzed with `tx-core` in wasm and enters the queue labeled
by origin (`extracted from recovery-batch.tar.gz`, `dropped file`,
`pasted`). An item whose hex exceeds the 1 MiB payload cap is flagged
`Invalid` at once, with its size and the cap in the reason.

A transaction already in the queue is never added twice. The same hex
reaches the queue easily by accident: pasted again, present in two
archives, or a file dropped a second time. Duplicates are matched on the
locally derived txid, so a re-encoded PSBT of an already-queued
transaction is caught too, and the first row wins. Pasting one is
reported inline ("That transaction is already in the queue.") rather than
silently clearing the box. `Invalid` rows carry no txid and so never
count as duplicates of each other.

Queue summary: a four-cell stat strip — "In queue" (total, default text),
"Clear the floor" (teal `#61ffe1`), "Below floor" (red `#ef445f`), "Fee
unknown" (muted `#a1a1a1`) — each an 11px uppercase eyebrow over a
28px/600 mono count, `padding:20px 26px`, separated by `#1f1f1f` right
borders (dropped on the last cell of each row). The strip is a grid of
`repeat(4,minmax(0,1fr)) minmax(190px,1.4fr)`, the fifth cell holding the
primary broadcast button flush right; on mobile it becomes
`repeat(2,minmax(0,1fr))` with hairlines under the first two cells and the
button spanning `1 / -1` full-width above a `#1f1f1f` top border. The
button (called "Send batch" throughout this document) renders a dynamic
label: `Sending…` while a run is in flight, `Send batch (n)` when more than
one item is submittable, otherwise `Send`. Every submittable row also
carries its own control, reading `Send` before an attempt and `Retry`
after a rejection or failure, so a queue of several can be worked through
one transaction at a time instead of only as a batch. It is disabled only while a run is in flight or nothing is
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

Rows carry contextual note cards (the warn/error triples in the tokens
above) only when there is something to say: "Missing UTXO data for n
input(s). Fee and final scripts could not be validated.", or
"Malformed: <reason>" / "Not valid base64 PSBT or hex transaction data."
A row that finalized cleanly gets no note. Its status, vsize and fee rate
already report that it is ready, and a card on every good row is clutter
that makes the rows which do need attention harder to spot.
(Not-fully-signed PSBTs never become rows — they are
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

Send behavior: "Send batch" iterates every submittable item —
each row that decoded successfully and has not already been accepted,
whatever its fee rate — strictly sequentially, in queue order. Each item's
finalized transaction hex, extracted locally by `tx-core` at queue time,
is sent as one `POST /api/transactions` straight to Slipstream; await the
response, then the next, preserving dependency order end-to-end. The next
request is only made once the previous one has fully returned, so a parent
always reaches Slipstream before a child spending it. Below-floor and
Fee-unknown rows
are submitted like any other; the fee labels informed the user, and the
user pressed the button (section 1). The only rows skipped are `Invalid`
ones, which have no transaction to send, and already-`Accepted` ones,
which must not be sent twice.

Outcomes never remove a row. A Slipstream rejection marks it `Rejected`
with the verbatim reason and the loop continues (an independent queue
shouldn't be stranded, and a dependent descendant of a rejected parent
fails on its own merits). A 429 is answered by retrying that same row in
place, since a rate limit means nothing was ever attempted for it: the row
goes `Rate limited` with a countdown note ticking once a second, then the
request is repeated, after 5, then 10, then 20 seconds. A `Retry-After`
value is used instead when the browser exposes it at all, clamped to
between 1 and 300 seconds because it comes from a host nobody here
controls, but it is usually invisible (section 2). A fourth 429 marks the
row `Failed` with "Slipstream is rate limiting this browser. Wait a few
minutes and send again." and pauses the run. A network failure
marks the row `Failed` with the same finality, pausing the run rather than
silently skipping. Every one of those rows keeps its txid, its analysis
and its place in the queue, so pressing "Send batch" again re-attempts
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

Answers 1 and 4 are taken verbatim. Answers 2 and 3 are the mockup's in
substance: "We operate no server and store nothing", and MARA learning
"the IP address it arrived from", describe this page exactly, because the
browser is what calls MARA. They are only tightened, to name where the
maths runs and to say that the finalized hex goes straight from the tab to
Slipstream (answer 2), and that the connection is the user's own, so MARA
sees the address they are browsing from and Tor or a VPN is the answer if
that matters (answer 3). Answer 5 is verbatim but for its closing clause:
the mockup says a transaction below the floor "needs to be rebuilt at a
higher rate before it can be submitted", which this UI no longer makes
true. It can be submitted, and between `submit_fee_rate` and
`effective_rate` it can even be accepted and then never mined (section 2),
so the answer reads that a lower-rate transaction may enter Slipstream's
private mempool but is not expected to be mined, and should be rebuilt at
a higher rate before the submission is relied on.

Footer: separated by a `#1a1a1a` top rule, "Built by
[Wizardsardine](https://wizardsardine.com)" on the left and the
"Disclosure" and "Slipstream terms" links on the right, 13px, wrapping on
narrow screens. Links are `#b0def0`, hovering to `#5fe7e4`, undecorated.

All authoritative validation is Slipstream's; the browser's analysis
exists for instant feedback, its structural checks to avoid wasting a
round trip, and its archive limits for tab self-protection. Nothing here
gates a submission on its fee rate (section 1). `unpack.rs` is pure (bytes
in, labeled text items out) and carries the archive and expansion test
suite under `wasm-bindgen-test` with tar/tgz/zip fixtures, including
entry-count and decompression-budget refusals (no crafted zip-bomb fixture
is built — the budget is exercised with a plainly oversized entry instead,
since the threat here is self-inflicted: a hostile archive can only
disrupt the tab of the person who dropped it). `slipstream.rs` keeps its
response handling in one pure `classify(status, body)` function so the
whole outcome ladder is testable without HTTP, against bodies recorded
from the live API (section 2).

### Deliberate departures from the mockup

The mockup wins on everything except the following eight points, places
where it simulates behavior it does not implement, or encodes a queue
policy that has since been reversed. Anything not on this list is a mockup
detail to be reproduced, not a decision to be revisited.

1. **Unfinalizable PSBTs are refused at load**, via the modal in
   section 1, rather than queued as a warn-note row. The mockup queues
   them with "Not fully signed: n of m inputs finalized" and would then
   submit them like anything else — doubly so now that nothing filters the
   broadcast run (point 5). The modal exists to close that hole; it is the
   one place this design does refuse on the user's behalf, and it does so
   because an unsigned transaction cannot succeed under any fee, not
   because we disagree with the user's judgment.
2. **The broadcast loop is strictly sequential** — one `POST`, awaited,
   then the next. The mockup fans out staggered `setTimeout`s 260 ms apart
   with overlapping lifetimes and invents a random txid; that cannot
   preserve the dependency ordering guaranteed in section 1, nor handle a
   429, nor surface a real rejection.
3. **The `· est.` fee suffix is not implemented.** The mockup marks every
   PSBT estimated because it approximates witness size at 108 B/input.
   `tx-core` finalizes and extracts before queueing, so every queued
   item's vsize is exact and the marker would never render.
4. **The paste-box button reads "Add to queue".** The mockup labels it
   `Broadcast`, which collides with the actual broadcast button beneath
   the stat strip — visibly a leftover from an earlier iteration.
5. **Nothing is filtered out of a broadcast run on fee grounds.** The
   mockup's `onBroadcastAll` submits only items whose derived rate clears
   the floor (`e.rate !== null && e.rate >= min`); this design submits
   every valid, not-yet-accepted row and lets Slipstream decide
   (section 1). The fee analysis is displayed, not enforced.
6. **Every row shows its txid from the moment it is queued**, not only
   after a successful broadcast as in the mockup (`hasTxid`, populated
   from a random hex string once its fake submission "succeeds"). The txid
   is derived locally from the finalized transaction, and a rejected or
   failed row keeps it — the user must always be able to walk away with
   the identifier of a transaction they built here.
7. **The fourth stat cell is labeled "Fee unknown", not "Needs input
   value".** With gating removed, nothing *needs* an input value; the
   field is there to inform. The mockup's label would now describe a
   requirement that does not exist.
8. **The fee card reads "Anything below this rate will not be mined",**
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

## 5. Deployment

A deployment is nginx serving static files, so all of it is three bash
scripts and two nginx snippets. There is no service to run, no
configuration to place on the host, and no secret to protect.

The scripts share the same skeleton: `set -euo pipefail`, colored
`log_info`/`log_warn`/`log_error` helpers and a `die` function, and
project-root resolution from `BASH_SOURCE` so each script works whether
invoked from the repo root or from `deploy/`. Each supports two modes:
with no argument it operates on the local machine; with a `user@host`
argument it rsyncs the project to `/opt/outofband/src` on the remote,
excluding build output, git data and tool state. It first creates the
directory over SSH, then re-executes itself there without arguments.
Nothing requires being run as root; `sudo` is invoked internally for
privileged steps only.

### install.sh — full bootstrap

Idempotent bootstrap of a fresh Debian/Ubuntu server, in order:

1. `apt-get install` prerequisites: `build-essential`, `pkg-config`,
   `curl`, `rsync`, `nginx`, `certbot`, `python3-certbot-nginx`.
2. Toolchain if missing: rustup at the pinned version, the
   `wasm32-unknown-unknown` target, and `trunk` (via `cargo install`).
3. Directories: `/opt/outofband` and `/var/www/outofband`.
4. Build: `trunk build --release` in `crates/broadcast-frontend`.
5. Install artifacts: the trunk `dist/` contents to `/var/www/outofband/`,
   with `rsync -a --delete` so a previous bundle leaves nothing behind.
6. nginx: install the application and security snippets, install the
   port-80 server wrapper when absent, migrate a pre-snippet site to the
   snippet include while preserving its `server_name`, symlink it into
   `sites-enabled/`, remove the default site, run `nginx -t`, and reload.
7. Health check: curl `http://127.0.0.1/` through nginx and fail loudly if
   it does not answer.
8. Print the remaining manual steps: set the real `server_name`, reload,
   run certbot.

TLS: the nginx config ships as a plain port-80 server so the first install
works before DNS or certificates exist. `install.sh` accepts an optional
`--domain example.com --email you@example.com` (valid only together); when
given and the domain already resolves to this host, it sets `server_name`
and runs `certbot --nginx -d <domain> --redirect --agree-tos -m <email>
-n`, which rewrites the site for 443 with the Let's Encrypt certificate
and installs the renewal timer. Without the flags, the README documents
the same one-liner to run by hand once DNS is ready. `server_name` is set
before certbot runs either way, because certbot picks the server block to
modify by matching it.

### update.sh — soft redeploy

Rsync the source (remote mode), rebuild the bundle, reinstall `dist/` and
the managed nginx snippets, validate and reload nginx, then re-run the
health check. It never touches certificates. A legacy Certbot site that
does not include the managed application snippet gets a migration warning
rather than being replaced, since replacing it would take the TLS
directives with it.

### clean.sh — teardown

Removes what install.sh created: the nginx site (available, and the
enabled symlink), both snippets, `/var/www/outofband` and
`/opt/outofband`. It also clears what an earlier backend deployment left
on a host that was installed before this design: the `broadcast-api`
service is stopped and disabled, its unit and binary deleted, and
`/etc/outofband`, which held the client code, removed. nginx is reloaded
only if it still validates, so an unrelated broken site cannot abort the
cleanup. The `outofband` system user is removed only after an explicit
`yes`. apt packages, the rust toolchain, certificates and the renewal
timer are left in place.

### nginx configuration

`deploy/nginx/outofband.conf` is a small port-80 server wrapper. It
includes `/etc/nginx/snippets/outofband-app.conf`, copied from
`deploy/nginx/outofband-app.conf`. The application snippet owns the static
root, the cache policy, the security headers and the logs. Keeping that
policy outside the server wrapper lets updates replace it without
overwriting the TLS directives Certbot adds to the wrapper.

There is nothing to proxy. `location /` serves `/var/www/outofband` with
`try_files $uri $uri/ /index.html`; `.wasm` and `.js` get an hour of
`public, immutable`, which is as long as it can safely be because trunk
builds with `filehash = false` and the bundle keeps its name across
deployments; `index.html` is `no-cache, no-store, must-revalidate`, so the
document that pulls in the bundle is always fresh. `autoindex` is off.
Security headers (`X-Content-Type-Options`, `X-Frame-Options`,
`X-XSS-Protection`) live in the second snippet and are included again
inside each nested cache-policy location, because an nginx `add_header` in
a child location replaces the inherited set instead of adding to it.

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

# Serve the frontend on :8080 over plain HTTP: no nginx, no TLS, no
# systemd, no root. It talks to MARA directly, so there is nothing else
# to start.
run:
    cd crates/broadcast-frontend && trunk serve --open

serve:
    cd crates/broadcast-frontend && trunk serve

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

`just run` is the everyday development command, and it is the whole
product: `trunk serve` on `127.0.0.1:8080` over plain HTTP, with a browser
opened. Nothing is installed, no port below 1024 is bound, no root is
needed, and TLS, which exists only in the nginx layer that certbot
rewrites, is simply absent. There is no config file to create and no
credential to fill in, so the fee card and sending behave in development
exactly as they do in production. That cuts both ways, and it is the one
thing to keep in mind: pressing Broadcast in development submits to the
real Slipstream. `SLIPSTREAM_BASE_URL` at build time points the page at
another host. `Trunk.toml` sets `filehash = false` and binds the dev
server to `127.0.0.1`, and carries no `[[proxy]]` entries at all, because
there is nothing to proxy to. The deploy scripts are never in the loop
while developing; `just local` remains available for rehearsing a real
install on this machine, and it is the only local command that wants
`sudo`.

`cargo clippy`, `cargo test` and `cargo fmt` at the workspace root cover
`tx-core` and everything in the frontend that compiles for the host,
including `slipstream.rs`'s response classifier and `queue.rs`'s analysis,
but only with `--workspace`: `default-members` is `tx-core` alone
(section 3), so a bare `cargo check` silently skips the crate that holds
most of the code. The frontend is additionally linted against the wasm
target, and `unpack.rs` runs under `wasm-bindgen-test` in a headless
browser, where extraction is exercised in the environment it actually runs
in.

## 6. Security and correctness notes

There is no server to attack. The deployment is nginx serving static
files, the only host that ever receives a transaction is MARA's, and the
page holds no credential, so the questions left are about the browser and
about the delivery.

The page holds no secret because it cannot: a wasm bundle is published by
definition, and anything compiled into it is public. This is only
tolerable because nothing the page does needs a credential (section 2). It
is also why no rotation procedure, no config permissions and no redaction
rules appear anywhere in this document: there is nothing to rotate,
protect or redact. A host installed before this design may still hold
`/etc/outofband/config.toml` with an old client code in it; `clean.sh`
removes it (section 5), and a code that has been deployed anywhere should
be treated as spent and rotated with MARA.

Input handling is strict before anything leaves the tab. An entry that
does not decode as a well-formed PSBT or transaction, or that decodes to a
transaction with no inputs or no outputs, or whose hex exceeds the 1 MiB
cap, becomes an `Invalid` row and is never submitted. That is not
validation, which is Slipstream's job; it is refusing to spend a round
trip on something that cannot succeed.

Archive parsing happens only in the browser, where a hostile file affects
only the tab of the user who dropped it. Even there, extraction streams
against a fixed 256 MiB budget, caps entry counts at 1,000, refuses nested
archives, and never touches a filesystem: tar path-traversal names are
inert labels, sanitized before display.

Nothing here can leak a transaction, because nothing here ever sees one:
the nginx access and error logs record requests for the page and its
assets, and a submission goes from the browser to MARA without passing
through this host at all. The flip side is the honest one: MARA sees the
user's IP address, because the connection is the user's own and nothing
proxies it. Tor or a VPN is the only answer, and the FAQ says so
(section 4). Rate limiting is MARA's alone, applied to the user's own
address rather than to a shared relay, which is why a `429` is a wait
rather than a queue-wide failure.

No system C library is linked anywhere; the dependency graph vendors C in
one place, libsecp256k1, compiled from source into the wasm (section 3).

nginx is the only thing listening, and it serves static files. UFW
guidance in the README: allow 22, 80 and 443, nothing else.

Honest limitations to document in the README rather than hide: nothing
here validates a fee. The displayed rate is analysis, Slipstream is the
only judge, and a below-floor transaction is submitted and bounced rather
than withheld. The displayed floor is additionally a polled value that can
lag MARA's real threshold by up to 30 seconds, so a transaction whose rate
looked comfortable can still bounce inside that window, and MARA's error
message says so. A rejection cannot be classified either: Slipstream
answers with prose, so "fee too low" and "consensus failure" reach the
user as MARA's own sentence rather than a typed error. CPFP is
unsupported: a low-fee parent is judged alone and rejected (section 2).
Queue behavior (submitting every valid row, keeping failed rows with their
txids, sequential ordering) is this page's own policy and binds nobody
else. The 1 MiB cap on queued transaction hex excludes the very largest
non-standard transactions (~8 MiB hex). And the page depends on MARA
continuing to serve permissive CORS and to accept submissions without a
client code: either can change without notice, and neither is something a
deploy on our side can fix.

## 7. Milestones

Task 0 (transcribing the endpoint URLs, response shapes and error codes
from the Slipstream API documentation) is **complete**: section 2 holds
the verified values, confirmed against the live API on 2026-08-02 and
re-probed on 2026-08-03. The milestones below are the narrative shape of
the work; the build itself was split into smaller reviewable changesets.

Task 1 builds `tx-core` with native unit tests plus a wasm build: format
detection, the PSBT finalization matrix (already finalized, finalizable,
incomplete), structural rejection of transactions with no inputs or no
outputs, vsize and fee computation, fee-rate comparison against a floor,
and txid derivation from a finalized transaction. Alongside it, the
frontend's `unpack.rs` under `wasm-bindgen-test` (tar/tgz/zip fixtures,
text-file expansion, ordering, the extraction limits).

Task 2 is `slipstream.rs`: the two calls, and the outcome ladder as a pure
function over `(status, body)`, tested against bodies recorded from the
live API, including a 200 carrying `"status":"error"`, an unparseable
body, and the rate-limit backoff schedule.

Task 3 is the Yew frontend, implemented to the section-4 specification and
verified against the mockup: design tokens and layout, the queue with live
`tx-core` analysis and a locally derived txid on every row, the
total-input-value field, drag-and-drop with in-browser extraction, and the
sequential broadcast loop with its rate-limit countdown and failure-pause
behavior, including a run where a below-floor item is submitted, rejected,
retained with its txid and reason, and successfully retried.

Task 4 is the deploy tooling, tested on a throwaway Debian VM from
`install.sh` through certbot to a successful broadcast, then `update.sh`
and `clean.sh`. Task 5 is documentation: README (deploy, certbot, log
locations, firewall, update and teardown, and the honest limitations).
