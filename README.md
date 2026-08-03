# Outofband

A relay in front of [MARA Slipstream](https://slipstream.mara.com): it
submits a finalized Bitcoin transaction directly to Slipstream, which mines
it without ever putting it in the public mempool. It exists for the case
where a key is known to be compromised (predictable RNG), and a normal
broadcast would lose the race to whoever else can spend those coins. A
single page lets a user paste or drop signed PSBTs and raw transactions,
see them analyzed locally, and broadcast them one at a time.

The service is an **open relay by design**: neither the browser nor the
server checks a transaction's fee against Slipstream's floor before
submitting it. See [Honest limitations](#honest-limitations) before relying
on this for anything.

## Development

`just run` starts the whole stack locally, over plain HTTP, with no nginx,
no TLS, no systemd, and no root: `broadcast-api` on `127.0.0.1:3010`, and
`trunk serve` on `:8080` in front of it, killing the backend when the
frontend process exits.

On first run it copies `deploy/config.toml` to `dev-config.toml` at the
repo root (gitignored, since it may hold a real client code) and prints a
reminder to fill in `client_code`. The page works before that: the fee
card polls `GET /fee`, which needs no credential, so the hero and the
queue's analysis all work; only pressing Broadcast fails, surfacing
Slipstream's own "Client codes are currently required to submit
transactions" message per row.

Other targets:

- `just clippy`: `cargo clippy`, plus `cargo clippy --target
  wasm32-unknown-unknown` in `crates/broadcast-frontend`.
- `just fmt`: `cargo fmt`, run at the workspace root and again in
  `crates/broadcast-frontend`.
- `just test`: `cargo test`, plus `wasm-pack test --headless --firefox`
  in `crates/broadcast-frontend`.
- `just serve`: frontend only (`trunk serve`); `:3010` must already be up.

## Deploy

Three scripts under `deploy/`, each runnable locally with no argument, or
against a remote host with `user@host` (rsyncs the project to
`/opt/outofband/src` and re-execs itself there over ssh):

- `just local` / `just deploy <user@host>` runs `deploy/install.sh`. Full
  bootstrap of a fresh Debian/Ubuntu host: installs apt prerequisites
  (`build-essential`, `pkg-config`, `curl`, `rsync`, `nginx`, `certbot`,
  `python3-certbot-nginx`), rustup + the `wasm32-unknown-unknown` target +
  `trunk` if missing, creates the `outofband` system user and
  `/etc/outofband`, `/opt/outofband`, `/var/www/outofband`, builds
  `broadcast-api` and the frontend, installs the binary, the frontend's
  `dist/`, the config (only if `/etc/outofband/config.toml` doesn't already
  exist, an existing config is never overwritten), the systemd unit and
  the nginx site, then runs a health check against `http://127.0.0.1/health`
  through nginx.
- `just update` / `just update-remote <host>` runs `deploy/update.sh`.
  Code-only redeploy: rebuilds the binary and the wasm frontend,
  reinstalls them plus the systemd unit and nginx site, restarts
  `broadcast-api`, reloads nginx. Never touches `/etc/outofband` or
  certificates. If the live nginx site already has `listen 443 ssl` (i.e.
  certbot has run), it leaves that file untouched rather than
  re-templating over the TLS config: diff `deploy/nginx/outofband.conf`
  against it by hand if routes or headers changed.
- `just clean-local` / `just clean-remote <host>` runs `deploy/clean.sh`.
  Stops and removes the service, the nginx site, the binary, and the
  install directories. Prompts with an explicit `yes` confirmation before
  removing `/etc/outofband`, since it holds the client code, and again
  before removing the `outofband` system user. Leaves apt packages and the
  Rust toolchain in place.

`install.sh` accepts `--domain example.com --email you@example.com`
(valid only together): if the domain already resolves to the host, it sets
`server_name` and runs certbot automatically. `install.sh` prints its
remaining manual steps at the end of the run; do them once it finishes:

- **If `--domain` wasn't passed, or DNS wasn't ready yet**: set the real
  `server_name` in `/etc/nginx/sites-available/outofband.conf` (it ships
  with the placeholder `outofband.example.com`), then reload nginx, then
  run certbot:

  ```
  sudo systemctl reload nginx
  sudo certbot --nginx -d <domain> --redirect --agree-tos -m <email> -n
  ```

  Set `server_name` first: `install.sh`'s own automatic certbot path sets
  it before invoking certbot for the same reason — certbot picks the
  server block to modify by matching `server_name`, so running it against
  the placeholder won't target the right site.

- **Always**: once you have a client code (see
  [Getting a client code](#getting-a-client-code)), fill in `client_code`
  in `/etc/outofband/config.toml` and restart the service — `install.sh`
  never fills this in, and the service can't submit transactions without
  it:

  ```
  sudo systemctl restart broadcast-api
  ```

The nginx site ships listening on plain port 80 so the first install works
before DNS or a certificate exist; certbot rewrites it for 443 with the
Let's Encrypt certificate and installs the renewal timer.

Check the service with `systemctl status broadcast-api`.

### Configuration

Everything lives in `/etc/outofband/config.toml`, templated from
`deploy/config.toml`:

| Section | Key | Default | Purpose |
|---|---|---|---|
| `[slipstream]` | `base_url` | `https://slipstream.mara.com` | Slipstream API root. |
| | `fee_endpoint` | `/api/rates` | Returns `effective_rate`; no credential needed. |
| | `submit_endpoint` | `/api/transactions` | Returns `{status, message}`. |
| | `client_code` | `""` | Secret. `root:outofband`, mode `640`. Required only to submit a transaction; fee rates work without it. |
| | `request_timeout_secs` | `30` | HTTP timeout for calls to Slipstream. |
| `[broadcast_api]` | `listen_addr` | `127.0.0.1:3010` | Backend bind address. Local only; nginx is the public surface. |
| | `fee_poll_secs` | `60` | How often the background task refreshes the fee cache. |
| | `rate_limit_window_secs` | `600` | Sliding-window length for the per-IP submission limiter. |
| | `rate_limit_max_tx` | `100` | Submissions an IP may make within that window. |
| | `max_payload_bytes` | `1048576` (1 MiB) | Cap on a `POST /broadcast` body, enforced by axum. |

`max_payload_bytes` excludes the largest non-standard transactions: a
maximal one serializes to roughly 4 MB, about 8 MiB as hex. Raising it to
`8388608` (8 MiB) to admit those is a change in three places, since nginx
must stay in step and neither `install.sh` nor `update.sh` derives its
nginx value from the live config file:

1. Edit `max_payload_bytes` directly in `/etc/outofband/config.toml` and
   restart the service (`sudo systemctl restart broadcast-api`).
   `install.sh` only writes this file when it's absent, so an existing
   deployment's config is never touched automatically.
2. Edit `MAX_PAYLOAD_BYTES` in `deploy/install.sh` and `deploy/update.sh`
   (both hardcode it, to derive nginx's `client_max_body_size`) so future
   installs and updates render a matching nginx value.
3. If the site is already certbot-managed (`listen 443 ssl` present in
   `/etc/nginx/sites-available/outofband.conf`), `update.sh` will not
   re-template that file: edit its `client_max_body_size` directive by
   hand and run `sudo nginx -t && sudo systemctl reload nginx`. On a
   pre-certbot site, the next `update.sh` picks up the new value
   automatically.

### Logs

Service logs: `journalctl -u broadcast-api`. The service currently logs
only lifecycle events (listen address on startup, bind/serve errors) to
stdout/stderr; it never logs a request body or transaction hex anywhere,
so nothing about a submitted transaction's contents reaches the logs.

nginx access and error logs: `/var/log/nginx/outofband-access.log` and
`/var/log/nginx/outofband-error.log`.

### Firewall

The backend binds to `127.0.0.1:3010` only; nginx is the sole public
surface. With UFW, allow 22, 80 and 443, and nothing else:

```
sudo ufw allow 22
sudo ufw allow 80
sudo ufw allow 443
```

### Getting a client code

Submitting a transaction (not fetching the fee rate) requires a Slipstream
client code. Contact `foundation@mara.com` to get one. Rotate it if it has
ever been pasted into an email, a chat, an issue tracker, or shell
history: whoever holds it can submit under this account, and MARA offers
no recourse for misuse. The deployed service (`/etc/outofband/config.toml`,
`root:outofband`, mode `640`) is the only place it should live.

## API note for scripted callers

`POST /broadcast` accepts exactly one finalized raw transaction hex per
request, and nothing else: no PSBT support, no batching, no multipart. A
PSBT must be finalized locally first, either with `bitcoin-cli
finalizepsbt` or by pasting it into the web page and copying the resulting
hex.

```
curl -s -X POST https://your-domain.example/broadcast \
  -H 'Content-Type: application/json' \
  -d '{"tx_hex": "0200000001..."}'
```

Response: `{"txid": "...", "vsize": n, "status":
"submitted"|"rejected"|"invalid", "error": "..."|null}`. HTTP status is
`400` for hex that doesn't decode to a consensus-valid transaction, `413`
over `max_payload_bytes`, `429` with `retry_after_secs` when rate limited,
`200` whenever Slipstream answered with a plain accept/reject on the
submission itself (`status: "submitted"`, or `status: "rejected"` for
e.g. a fee-too-low bounce, surfaced verbatim in `error`), and `502` both
when Slipstream is unreachable *and* when Slipstream's response indicates
a client-code problem (missing or rejected credential) — that case also
comes back as `status: "rejected"`, so a `200`/`"rejected"` pair is not
the only shape an "answered" submission can take; a scripted caller
should check the HTTP status, not just the body's `status` field, before
concluding the submission itself was evaluated.

For several transactions, loop and submit each finalized hex in turn:

```
for f in tx1.hex tx2.hex tx3.hex; do
  curl -s -X POST https://your-domain.example/broadcast \
    -H 'Content-Type: application/json' \
    -d "{\"tx_hex\": \"$(cat "$f")\"}"
  echo
done
```

`GET /fee` needs no credential and returns the cached floor:
`{"effective_rate_sat_vb": n, "age_secs": n, "stale": bool}` (`503` if no
successful poll has happened yet). `GET /health` is a bare liveness check.

## Honest limitations

- **Nothing validates a fee.** Not the browser, not the server. A
  below-floor transaction is submitted and bounced by Slipstream rather
  than withheld, and the bounce consumes a rate-limit slot the same as any
  other submission.
- **The displayed floor is cached.** It can lag Slipstream's real threshold
  by up to `fee_poll_secs`, so a transaction that looked fine when queued
  can still bounce.
- **Rejections aren't classified.** Slipstream answers with prose, so
  "fee too low" and "consensus failure" both reach the user as MARA's own
  sentence.
- **CPFP is unsupported.** A low-fee parent submitted alone is judged and
  rejected on its own rate. Slipstream has a package endpoint for this;
  this service doesn't use it, so a parent that depends on its child's fee
  has to go through MARA directly.
- **Queue behavior is frontend policy, not a server guarantee.** Retry,
  ordering, and keeping failed rows visible all happen in the browser; the
  server only ever sees one transaction at a time.
- **The default 1 MiB payload cap excludes the largest non-standard
  transactions** unless raised (see [Configuration](#configuration)).
- **The service is an open relay by design.** The only protections are the
  per-IP rate-limit budget, nginx's `limit_req`, and the payload cap.
- **MARA offers no support and no recourse.** An accepted transaction that
  was built wrong, or paid the wrong fee, cannot be helped after the fact.

## Links

- [Coldcard RNG vulnerability, and why you might need this tool](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)
- [MARA Slipstream](https://slipstream.mara.com): the service this relay
  submits to; its terms and no-support policy are stated on that site.
