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
    cd crates/broadcast-frontend && trunk serve   # same as `run`, without opening a browser

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
