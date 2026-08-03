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

REMOTE="${1:-}"
[ $# -le 1 ] || die "unexpected argument: $2"

if [ -n "$REMOTE" ]; then
  REMOTE_USER="${REMOTE%%@*}"
  [ "$REMOTE_USER" != "$REMOTE" ] || die "expected user@host, got: $REMOTE"

  log_info "preparing $REMOTE"
  ssh "$REMOTE" "sudo mkdir -p /opt/outofband/src && sudo chown ${REMOTE_USER}:${REMOTE_USER} /opt/outofband/src"

  log_info "syncing source to $REMOTE:/opt/outofband/src"
  rsync -az --delete --exclude target/ --exclude .git/ "$PROJECT_ROOT"/ "$REMOTE":/opt/outofband/src/

  log_info "running clean.sh on $REMOTE"
  # -t allocates a pty so the /etc/outofband confirmation prompt below works interactively
  ssh -t "$REMOTE" "/opt/outofband/src/deploy/clean.sh"

  log_info "remote clean complete"
  exit 0
fi

if sudo systemctl disable --now broadcast-api 2>/dev/null; then
  log_info "stopped and disabled broadcast-api"
else
  log_warn "broadcast-api service not found or already stopped"
fi
sudo rm -f /etc/systemd/system/broadcast-api.service
sudo systemctl daemon-reload

sudo rm -f /etc/nginx/sites-enabled/outofband.conf /etc/nginx/sites-available/outofband.conf /etc/nginx/conf.d/outofband-zone.conf
if command -v nginx >/dev/null 2>&1; then
  sudo nginx -t
  sudo systemctl reload nginx
fi

sudo rm -f /usr/local/bin/broadcast-api
sudo rm -rf /var/www/outofband /opt/outofband

log_warn "/etc/outofband holds the Slipstream client code and is not removed automatically."
CONFIRM=""
read -r -p "Type 'yes' to also remove /etc/outofband: " CONFIRM || true
if [ "$CONFIRM" = "yes" ]; then
  sudo rm -rf /etc/outofband
  log_info "removed /etc/outofband"
else
  log_warn "/etc/outofband left in place"
fi

if id outofband >/dev/null 2>&1; then
  CONFIRM=""
  read -r -p "Type 'yes' to also remove the outofband system user: " CONFIRM || true
  if [ "$CONFIRM" = "yes" ]; then
    sudo userdel outofband
    log_info "removed outofband system user"
  else
    log_warn "outofband system user left in place"
  fi
else
  log_warn "outofband system user not found, nothing to remove"
fi

log_info "apt packages, the rust toolchain, TLS certificates and the certbot renewal timer are left in place"
log_info "clean complete"
