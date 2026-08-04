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

# Resolved here because the rsync below excludes .git, leaving the server with
# no repository to ask. Exported so the frontend build bakes it in.
if [ -z "${OUTOFBAND_COMMIT:-}" ] && git -C "$PROJECT_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  OUTOFBAND_COMMIT="$(git -C "$PROJECT_ROOT" rev-parse --short HEAD)"
  [ -n "$(git -C "$PROJECT_ROOT" status --porcelain)" ] && OUTOFBAND_COMMIT="$OUTOFBAND_COMMIT-dirty"
fi
export OUTOFBAND_COMMIT="${OUTOFBAND_COMMIT:-unknown}"

REMOTE="${1:-}"
[ $# -le 1 ] || die "unexpected argument: $2"

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

  log_info "running update.sh on $REMOTE"
  ssh -- "$REMOTE" "OUTOFBAND_COMMIT=$(printf '%q' "$OUTOFBAND_COMMIT") /opt/outofband/src/deploy/update.sh"

  log_info "remote update complete"
  exit 0
fi

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

log_info "building broadcast-frontend"
# Wipe dist first: trunk leaves stale artifacts behind on an interrupted
# build, and the rsync below is --delete, so a half-built dist publishes a
# blank page and reports success.
rm -rf "$PROJECT_ROOT/crates/broadcast-frontend/dist"
(cd "$PROJECT_ROOT/crates/broadcast-frontend" && trunk build --release)

log_info "installing frontend assets"
sudo rsync -a --delete "$PROJECT_ROOT/crates/broadcast-frontend/dist/" /var/www/outofband/

log_info "installing nginx snippets"
sudo mkdir -p /etc/nginx/snippets
# Rendered, not copied: the CSP names the inline scripts of the page installed
# just above by their hash, so it has to be built from that exact page.
"$PROJECT_ROOT/deploy/render-security-headers.sh" \
  "$PROJECT_ROOT/deploy/nginx/outofband-security-headers.conf.in" \
  "$PROJECT_ROOT/crates/broadcast-frontend/dist/index.html" \
  | sudo tee /etc/nginx/snippets/outofband-security-headers.conf >/dev/null
sudo cp "$PROJECT_ROOT/deploy/nginx/outofband-app.conf" /etc/nginx/snippets/outofband-app.conf

NGINX_SITE=/etc/nginx/sites-available/outofband.conf
if [ -f "$NGINX_SITE" ] && sudo grep -qF 'include /etc/nginx/snippets/outofband-app.conf;' "$NGINX_SITE"; then
  log_info "managed nginx application snippet is active"
elif [ -f "$NGINX_SITE" ] && sudo grep -q "listen 443 ssl" "$NGINX_SITE"; then
  log_warn "legacy TLS site does not include /etc/nginx/snippets/outofband-app.conf; managed application updates are not active until the site is migrated"
else
  SERVER_NAME="outofband.wizardsardine.com"
  if [ -f "$NGINX_SITE" ]; then
    EXISTING_NAME="$(sudo awk '$1 == "server_name" { print $2 }' "$NGINX_SITE" | tr -d ';' | head -n1)"
    [ -n "$EXISTING_NAME" ] && SERVER_NAME="$EXISTING_NAME"
  fi
  log_info "installing nginx site conf to $NGINX_SITE"
  sed -e "s/server_name outofband.wizardsardine.com;/server_name ${SERVER_NAME};/" \
      "$PROJECT_ROOT/deploy/nginx/outofband.conf" | sudo tee "$NGINX_SITE" >/dev/null
fi

log_info "reloading nginx"
sudo nginx -t
sudo systemctl reload nginx

log_info "checking the site responds"
# Resolve the served name to loopback rather than requesting 127.0.0.1: once
# certbot rewrites the site, a request whose Host matches no server_name gets
# a 404, and the old check failed on a perfectly healthy deploy. -L follows
# the 80 to 443 redirect, so this exercises the certificate too.
SERVED_NAME="$(sudo awk '$1 == "server_name" { print $2 }' "$NGINX_SITE" 2>/dev/null | tr -d ';' | head -n1)"
[ -n "$SERVED_NAME" ] || SERVED_NAME="outofband.wizardsardine.com"
curl -sfL --max-time 15 \
  --resolve "$SERVED_NAME:80:127.0.0.1" \
  --resolve "$SERVED_NAME:443:127.0.0.1" \
  "http://$SERVED_NAME/" >/dev/null || die "$SERVED_NAME did not respond on this host"
log_info "site check passed"

log_info "update complete"
