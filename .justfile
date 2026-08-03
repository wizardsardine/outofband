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
