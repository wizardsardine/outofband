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

DOMAIN=""
EMAIL=""
REMOTE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --domain)
      [ $# -ge 2 ] || die "--domain requires a value"
      DOMAIN="$2"
      shift 2
      ;;
    --email)
      [ $# -ge 2 ] || die "--email requires a value"
      EMAIL="$2"
      shift 2
      ;;
    --*)
      die "unknown flag: $1"
      ;;
    *)
      [ -z "$REMOTE" ] || die "unexpected argument: $1"
      REMOTE="$1"
      shift
      ;;
  esac
done

if [ -n "$DOMAIN" ] && [ -z "$EMAIL" ]; then
  die "--domain requires --email"
fi
if [ -z "$DOMAIN" ] && [ -n "$EMAIL" ]; then
  die "--email requires --domain"
fi

if [ -n "$REMOTE" ]; then
  REMOTE_USER="${REMOTE%%@*}"
  [ "$REMOTE_USER" != "$REMOTE" ] || die "expected user@host, got: $REMOTE"

  log_info "preparing $REMOTE"
  ssh "$REMOTE" "sudo mkdir -p /opt/outofband/src && sudo chown ${REMOTE_USER}:${REMOTE_USER} /opt/outofband/src"

  log_info "syncing source to $REMOTE:/opt/outofband/src"
  rsync -az --delete --exclude target/ --exclude .git/ "$PROJECT_ROOT"/ "$REMOTE":/opt/outofband/src/

  if [ -n "$DOMAIN" ]; then
    log_warn "TLS flags are not forwarded through the remote re-exec; once this completes, run: ssh $REMOTE '/opt/outofband/src/deploy/install.sh --domain $DOMAIN --email $EMAIL'"
  fi

  log_info "running install.sh on $REMOTE"
  ssh "$REMOTE" "/opt/outofband/src/deploy/install.sh"

  log_info "remote install complete"
  exit 0
fi

export DEBIAN_FRONTEND=noninteractive

log_info "installing apt prerequisites"
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl rsync nginx certbot python3-certbot-nginx

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

if ! command -v rustup >/dev/null 2>&1; then
  log_info "installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.88
  source "$HOME/.cargo/env"
fi

log_info "adding wasm32-unknown-unknown target"
(cd "$PROJECT_ROOT" && rustup target add wasm32-unknown-unknown)

if ! command -v trunk >/dev/null 2>&1; then
  log_info "installing trunk"
  cargo install trunk
fi

log_info "creating system user and directories"
id outofband >/dev/null 2>&1 || sudo useradd --system --no-create-home --shell /usr/sbin/nologin outofband
sudo mkdir -p /etc/outofband /opt/outofband /var/www/outofband

log_info "building broadcast-api"
(cd "$PROJECT_ROOT" && cargo build --release -p broadcast-api)

log_info "building broadcast-frontend"
(cd "$PROJECT_ROOT/crates/broadcast-frontend" && trunk build --release)

log_info "installing artifacts"
sudo install -m 755 "$PROJECT_ROOT/target/release/broadcast-api" /usr/local/bin/broadcast-api
sudo rsync -a --delete "$PROJECT_ROOT/crates/broadcast-frontend/dist/" /var/www/outofband/

if [ ! -f /etc/outofband/config.toml ]; then
  log_info "installing default config to /etc/outofband/config.toml"
  sed "s/^max_payload_bytes = [0-9]*/max_payload_bytes = ${MAX_PAYLOAD_BYTES}/" \
    "$PROJECT_ROOT/deploy/config.toml" | sudo tee /etc/outofband/config.toml >/dev/null
  sudo chown root:outofband /etc/outofband/config.toml
  sudo chmod 640 /etc/outofband/config.toml
else
  log_info "config already exists at /etc/outofband/config.toml, leaving it untouched"
fi

log_info "installing systemd unit"
sudo cp "$PROJECT_ROOT/deploy/systemd/broadcast-api.service" /etc/systemd/system/broadcast-api.service
sudo systemctl daemon-reload
sudo systemctl enable --now broadcast-api

log_info "installing nginx zone snippet"
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-zone.conf" /etc/nginx/conf.d/outofband-zone.conf

if [ ! -f /etc/nginx/sites-available/outofband.conf ]; then
  log_info "installing nginx site to /etc/nginx/sites-available/outofband.conf"
  sed "s/client_max_body_size 2m;/client_max_body_size ${NGINX_CLIENT_MAX_BODY};/" \
    "$PROJECT_ROOT/deploy/nginx/outofband.conf" | sudo tee /etc/nginx/sites-available/outofband.conf >/dev/null
else
  log_info "nginx site already exists at /etc/nginx/sites-available/outofband.conf, leaving it untouched"
fi
sudo ln -sf /etc/nginx/sites-available/outofband.conf /etc/nginx/sites-enabled/outofband.conf
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx

log_info "checking health endpoint"
curl -sf http://127.0.0.1/health >/dev/null || die "health check failed: http://127.0.0.1/health did not respond"
log_info "health check passed"

CERT_ISSUED=0
if [ -n "$DOMAIN" ]; then
  DOMAIN_IP="$(getent hosts "$DOMAIN" 2>/dev/null | awk '{print $1}' | head -n1 || true)"
  PUBLIC_IP="$(curl -sf --max-time 5 https://api.ipify.org || true)"
  if [ -n "$DOMAIN_IP" ] && [ -n "$PUBLIC_IP" ] && [ "$DOMAIN_IP" = "$PUBLIC_IP" ]; then
    log_info "$DOMAIN resolves to this host, requesting a certificate"
    sudo sed -i "s/server_name outofband.example.com;/server_name ${DOMAIN};/" /etc/nginx/sites-available/outofband.conf
    sudo nginx -t
    sudo systemctl reload nginx
    sudo certbot --nginx -d "$DOMAIN" --redirect --agree-tos -m "$EMAIL" -n
    CERT_ISSUED=1
  else
    log_warn "$DOMAIN does not resolve to this host yet, skipping certbot"
  fi
fi

log_info "install complete, remaining manual steps:"
log_info "  - set the real server_name in /etc/nginx/sites-available/outofband.conf, then: sudo systemctl reload nginx"
log_info "  - fill client_code in /etc/outofband/config.toml, then: sudo systemctl restart broadcast-api"
if [ "$CERT_ISSUED" -eq 0 ]; then
  log_info "  - once DNS points at this host: sudo certbot --nginx -d <domain> --redirect --agree-tos -m <email> -n"
fi
