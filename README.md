# Outofband

A single static page in front of [MARA
Slipstream](https://slipstream.mara.com): paste or drop signed PSBTs and
raw transactions, see them decoded, finalized and priced locally, then
submit them one at a time to Slipstream, which mines them without ever
putting them in the public mempool. It is useful to a narrow set of users
— mainly Liana, Miniscript and some multisig wallets — for whom keeping a
spend out of the public mempool changes the outcome. It is *not* the
recommended route for a wallet critically at risk: the [Coldcard RNG
write-up](https://wizardsardine.com/blog/coldcard-rng-vulnerability/) says
what to do in that case, and the page says so at the top.

There is no backend. Parsing, finalization, fee maths and the submission
itself all happen in the browser, and the submission goes straight from
the tab to `slipstream.mara.com`. A deployment is nginx serving a wasm
bundle and nothing else, so nothing pasted into the page ever reaches this
project's host. See [Honest limitations](#honest-limitations) before
relying on this for anything.

## Languages

Six catalogues live in `crates/broadcast-frontend/src/i18n/`, one file per
language. `Strings` is a struct, so adding a string will not compile until
every language has one; that is the point.

Only English is offered today. The other five are LLM drafts marked
unreviewed in `Lang::reviewed`, and the switcher hides a language until that
flag flips. Flip it in the same commit that records who reviewed it. A
mistranslated "do not press the broadcast button" costs someone their coins,
which is the whole reason for the gate.

To read a draft before it is signed off, flip its `reviewed` arm locally and
`just run`.

## Development

`just run` serves the page on `127.0.0.1:8080` over plain HTTP with `trunk
serve`, and opens a browser; `just serve` is the same without opening one.
There is nothing else to start, and no credential to fill in: the page
talks to MARA directly, so the fee card and sending behave in dev
exactly as they do in production. Sending against the real
Slipstream is a real submission.

`SLIPSTREAM_BASE_URL` at build time points the page at another host; with
it unset the page uses `https://slipstream.mara.com`.

Other targets:

- `just clippy`: `cargo clippy`, plus `cargo clippy --target
  wasm32-unknown-unknown` in `crates/broadcast-frontend`.
- `just fmt`: `cargo fmt`, run at the workspace root and again in
  `crates/broadcast-frontend`.
- `just test`: `cargo test`, plus `wasm-pack test --headless --firefox`
  in `crates/broadcast-frontend`.

## Deploy

Three scripts under `deploy/`, each runnable locally with no argument, or
against a remote host with `user@host` (rsyncs the project to
`/opt/outofband/src` and re-execs itself there over ssh):

- `just local` / `just deploy <user@host>` runs `deploy/install.sh`. Full
  bootstrap of a fresh Debian/Ubuntu host: installs apt prerequisites
  (`build-essential`, `clang`, `pkg-config`, `curl`, `rsync`, `nginx`, `certbot`,
  `python3-certbot-nginx`), rustup + the `wasm32-unknown-unknown` target +
  a pinned `trunk` if missing or out of date, creates `/opt/outofband` and
  `/var/www/outofband`,
  builds the frontend with `trunk build --release`, installs its `dist/`
  to `/var/www/outofband`, installs the nginx site and its snippets, then
  runs a health check against `http://127.0.0.1/` through nginx.
- `just update` / `just update-remote <host>` runs `deploy/update.sh`.
  Rebuilds the bundle, reinstalls it plus the nginx snippets, reloads
  nginx. Never touches certificates. Application routes, caching and
  headers live in `/etc/nginx/snippets/outofband-app.conf`, which updates
  without replacing Certbot's TLS server block. A deployment created
  before this split needs a one-time migration: preserve its TLS and
  `server_name` directives and replace its old application locations with
  `include /etc/nginx/snippets/outofband-app.conf;`.
- `just clean-local` / `just clean-remote <host>` runs `deploy/clean.sh`.
  Removes the nginx site, its snippets, and the install directories, and
  clears out what an earlier backend deployment left behind, prompting
  with an explicit `yes` confirmation before removing the `outofband`
  system user. Leaves apt packages, the Rust toolchain, TLS certificates
  and the renewal timer in place.

Both the domain and the certbot contact default to
`outofband.wizardsardine.com` and `contact@wizardsardine.com`, and the TLS
flags are forwarded through the remote re-exec, so a bare deploy issues the
certificate:

```
./deploy/install.sh user@host
```

Pass `--domain <host>` to deploy under a different name, `--email
<address>` to register the account elsewhere. Certbot runs only when the
domain already resolves to an address on the host, in either family, so a
certificate is never requested for a name that cannot answer the
challenge. Otherwise do it by hand once DNS points at the host, then
reload nginx and run certbot:

```
sudo systemctl reload nginx
sudo certbot --nginx -d outofband.wizardsardine.com --redirect --agree-tos -m <email> -n
```

Set `server_name` first, for the same reason `install.sh` sets it before
invoking certbot itself: certbot picks the server block to modify by
matching `server_name`, so running it against the placeholder won't target
the right site.

The nginx site ships listening on plain port 80 so the first install works
before DNS or a certificate exist; certbot rewrites it for 443 with the
Let's Encrypt certificate and installs the renewal timer.

### Logs

nginx access and error logs: `/var/log/nginx/outofband-access.log` and
`/var/log/nginx/outofband-error.log`. They record requests for the page
and its assets and nothing else: a submission goes from the browser to
MARA without passing through this host, so no transaction hex exists in
any log here to leak.

### Firewall

nginx is the only thing listening, and it only serves static files. With
UFW, allow 22, 80 and 443, and nothing else:

```
sudo ufw allow 22
sudo ufw allow 80
sudo ufw allow 443
```

## Honest limitations

- **Rejections aren't classified.** Slipstream answers with prose, so
  "fee too low" and "consensus failure" both reach the user as MARA's own
  sentence.
- **CPFP is unsupported.** A low-fee parent submitted alone is judged and
  rejected on its own rate. Slipstream has a package endpoint for this;
  the page doesn't use it, so a parent that depends on its child's fee
  has to go through MARA directly.
- **The browser talks to MARA directly.** MARA sees the user's IP address,
  because the connection is the user's own and nothing here proxies it;
  Tor or a VPN is the only answer to that. It also means MARA can break
  this site with no deploy on our side, by tightening CORS on
  `/api/transactions` or moving the endpoint.

## Links

- [Coldcard RNG vulnerability, and why you might need this tool](https://wizardsardine.com/blog/coldcard-rng-vulnerability/)
- [MARA Slipstream](https://slipstream.mara.com): the service this page
  submits to; its terms and no-support policy are stated on that site.
