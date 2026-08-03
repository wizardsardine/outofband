#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*" >&2; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
die() { log_error "$*"; exit 1; }

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Maximum transaction-hex length. nginx also allows JSON framing overhead.
MAX_PAYLOAD_BYTES=1048576
NGINX_JSON_OVERHEAD_BYTES=13
NGINX_CLIENT_MAX_BODY="$(( (MAX_PAYLOAD_BYTES + NGINX_JSON_OVERHEAD_BYTES + 1048575) / 1048576 ))m"

REMOTE="${1:-}"
[ $# -le 1 ] || die "unexpected argument: $2"

if [ -n "$REMOTE" ]; then
  REMOTE_USER="${REMOTE%%@*}"
  [ "$REMOTE_USER" != "$REMOTE" ] || die "expected user@host, got: $REMOTE"

  log_info "preparing $REMOTE"
  ssh "$REMOTE" "sudo mkdir -p /opt/outofband/src && sudo chown ${REMOTE_USER}:${REMOTE_USER} /opt/outofband/src"

  log_info "syncing source to $REMOTE:/opt/outofband/src"
  rsync -az --delete --exclude target/ --exclude .git/ "$PROJECT_ROOT"/ "$REMOTE":/opt/outofband/src/

  log_info "running update.sh on $REMOTE"
  ssh "$REMOTE" "/opt/outofband/src/deploy/update.sh"

  log_info "remote update complete"
  exit 0
fi

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

log_info "building broadcast-api"
(cd "$PROJECT_ROOT" && cargo build --release -p broadcast-api)

log_info "building broadcast-frontend"
(cd "$PROJECT_ROOT/crates/broadcast-frontend" && trunk build --release)

log_info "installing binary"
sudo install -m 755 "$PROJECT_ROOT/target/release/broadcast-api" /usr/local/bin/broadcast-api

log_info "installing frontend assets"
sudo rsync -a --delete "$PROJECT_ROOT/crates/broadcast-frontend/dist/" /var/www/outofband/

log_info "installing systemd unit"
sudo cp "$PROJECT_ROOT/deploy/systemd/broadcast-api.service" /etc/systemd/system/broadcast-api.service
sudo systemctl daemon-reload

log_info "installing nginx snippets"
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-zone.conf" /etc/nginx/conf.d/outofband-zone.conf
sudo mkdir -p /etc/nginx/snippets
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-security-headers.conf" /etc/nginx/snippets/outofband-security-headers.conf
sed "s/client_max_body_size 2m;/client_max_body_size ${NGINX_CLIENT_MAX_BODY};/" \
  "$PROJECT_ROOT/deploy/nginx/outofband-app.conf" | sudo tee /etc/nginx/snippets/outofband-app.conf >/dev/null

NGINX_SITE=/etc/nginx/sites-available/outofband.conf
if [ -f "$NGINX_SITE" ] && sudo grep -qF 'include /etc/nginx/snippets/outofband-app.conf;' "$NGINX_SITE"; then
  log_info "managed nginx application snippet is active"
elif [ -f "$NGINX_SITE" ] && sudo grep -q "listen 443 ssl" "$NGINX_SITE"; then
  log_warn "legacy TLS site does not include /etc/nginx/snippets/outofband-app.conf; managed application updates are not active until the site is migrated"
else
  SERVER_NAME="outofband.example.com"
  if [ -f "$NGINX_SITE" ]; then
    EXISTING_NAME="$(sudo awk '$1 == "server_name" { print $2 }' "$NGINX_SITE" | tr -d ';' | head -n1)"
    [ -n "$EXISTING_NAME" ] && SERVER_NAME="$EXISTING_NAME"
  fi
  log_info "installing nginx site conf to $NGINX_SITE"
  sed -e "s/client_max_body_size 2m;/client_max_body_size ${NGINX_CLIENT_MAX_BODY};/" \
      -e "s/server_name outofband.example.com;/server_name ${SERVER_NAME};/" \
      "$PROJECT_ROOT/deploy/nginx/outofband.conf" | sudo tee "$NGINX_SITE" >/dev/null
fi

log_info "restarting broadcast-api"
sudo systemctl restart broadcast-api

log_info "reloading nginx"
sudo nginx -t
sudo systemctl reload nginx

log_info "checking health endpoint"
curl -sf http://127.0.0.1/health >/dev/null || die "health check failed: http://127.0.0.1/health did not respond"
log_info "health check passed"

log_info "update complete"
