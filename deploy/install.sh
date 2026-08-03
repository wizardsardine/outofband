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
  ssh -- "$REMOTE" 'sudo mkdir -p /opt/outofband/src && sudo chown "$(id -un):$(id -gn)" /opt/outofband/src'

  log_info "syncing source to $REMOTE:/opt/outofband/src"
  rsync -az --delete --protect-args \
    --exclude '/.git/' \
    --exclude '/.claude/' \
    --exclude '/.cm/' \
    --exclude 'dist/' \
    --exclude 'target/' \
    -- "$PROJECT_ROOT"/ "$REMOTE":/opt/outofband/src/

  if [ -n "$DOMAIN" ]; then
    log_warn "TLS flags are not forwarded through the remote re-exec; once this completes, run: ssh $REMOTE '/opt/outofband/src/deploy/install.sh --domain $DOMAIN --email $EMAIL'"
  fi

  log_info "running install.sh on $REMOTE"
  ssh -- "$REMOTE" /opt/outofband/src/deploy/install.sh

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

log_info "creating directories"
sudo mkdir -p /opt/outofband /var/www/outofband

log_info "building broadcast-frontend"
(cd "$PROJECT_ROOT/crates/broadcast-frontend" && trunk build --release)

log_info "installing artifacts"
sudo rsync -a --delete "$PROJECT_ROOT/crates/broadcast-frontend/dist/" /var/www/outofband/

log_info "installing nginx snippets"
sudo mkdir -p /etc/nginx/snippets
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-security-headers.conf" /etc/nginx/snippets/outofband-security-headers.conf
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-app.conf" /etc/nginx/snippets/outofband-app.conf

if [ ! -f /etc/nginx/sites-available/outofband.conf ]; then
  log_info "installing nginx site to /etc/nginx/sites-available/outofband.conf"
  sudo cp "$PROJECT_ROOT/deploy/nginx/outofband.conf" /etc/nginx/sites-available/outofband.conf
elif sudo grep -qF 'include /etc/nginx/snippets/outofband-app.conf;' /etc/nginx/sites-available/outofband.conf; then
  log_info "managed nginx application snippet is active"
elif sudo grep -q "listen 443 ssl" /etc/nginx/sites-available/outofband.conf; then
  log_warn "legacy TLS site does not include /etc/nginx/snippets/outofband-app.conf; managed application updates are not active until the site is migrated"
else
  SERVER_NAME="$(sudo awk '$1 == "server_name" { print $2 }' /etc/nginx/sites-available/outofband.conf | tr -d ';' | head -n1)"
  [ -n "$SERVER_NAME" ] || SERVER_NAME="outofband.example.com"
  log_info "migrating nginx site to the managed application snippet"
  sed "s/server_name outofband.example.com;/server_name ${SERVER_NAME};/" \
    "$PROJECT_ROOT/deploy/nginx/outofband.conf" | sudo tee /etc/nginx/sites-available/outofband.conf >/dev/null
fi
sudo ln -sf /etc/nginx/sites-available/outofband.conf /etc/nginx/sites-enabled/outofband.conf
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx

log_info "checking the site responds"
curl -sf http://127.0.0.1/ >/dev/null || die "http://127.0.0.1/ did not respond"
log_info "site check passed"

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
if [ "$CERT_ISSUED" -eq 0 ]; then
  log_info "  - once DNS points at this host: sudo certbot --nginx -d <domain> --redirect --agree-tos -m <email> -n"
fi
