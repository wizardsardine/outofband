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

# Both default, so a bare install issues a certificate. The flags exist to
# deploy under a different name or register the account elsewhere.
DEFAULT_DOMAIN="outofband.wizardsardine.com"
DEFAULT_EMAIL="contact@wizardsardine.com"
[ -n "$DOMAIN" ] || DOMAIN="$DEFAULT_DOMAIN"
[ -n "$EMAIL" ] || EMAIL="$DEFAULT_EMAIL"

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

  # The remote runs through a shell, so the flags are requoted rather than
  # interpolated raw: certbot has to run on the host that answers the ACME
  # challenge, and skipping it here would leave the site on plain HTTP.
  REMOTE_CMD="OUTOFBAND_COMMIT=$(printf '%q' "$OUTOFBAND_COMMIT") /opt/outofband/src/deploy/install.sh --domain $(printf '%q' "$DOMAIN") --email $(printf '%q' "$EMAIL")"

  log_info "running install.sh on $REMOTE"
  ssh -- "$REMOTE" "$REMOTE_CMD"

  log_info "remote install complete"
  exit 0
fi

export DEBIAN_FRONTEND=noninteractive

log_info "installing apt prerequisites"
sudo apt-get update
# clang is not optional: `bitcoin` pulls `secp256k1-sys`, which compiles
# libsecp256k1 from C, and cc-rs targets wasm32 with clang only. gcc from
# build-essential cannot, so the frontend build dies in the wasm step.
sudo apt-get install -y build-essential clang pkg-config curl rsync nginx certbot python3-certbot-nginx

[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

if ! command -v rustup >/dev/null 2>&1; then
  log_info "installing rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.88
  source "$HOME/.cargo/env"
fi

log_info "adding wasm32-unknown-unknown target"
(cd "$PROJECT_ROOT" && rustup target add wasm32-unknown-unknown)

# Pinned and --locked: a bare `cargo install trunk` re-resolves every
# transitive dependency, and a semver-compatible cssparser release breaks
# lightningcss, so an unpinned install fails on a box that resolves it
# today even though the same command worked last month.
TRUNK_VERSION="0.21.14"
if ! command -v trunk >/dev/null 2>&1; then
  log_info "installing trunk $TRUNK_VERSION"
  cargo install --locked "trunk@$TRUNK_VERSION"
elif [ "$(trunk --version | awk '{print $2}')" != "$TRUNK_VERSION" ]; then
  log_info "updating trunk to $TRUNK_VERSION"
  cargo install --locked --force "trunk@$TRUNK_VERSION"
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
  [ -n "$SERVER_NAME" ] || SERVER_NAME="outofband.wizardsardine.com"
  log_info "migrating nginx site to the managed application snippet"
  sed "s/server_name outofband.wizardsardine.com;/server_name ${SERVER_NAME};/" \
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
{
  # Compare every address the name resolves to, both families, against every
  # global address on this host. Matching a single resolver-preferred entry
  # against an IPv4-only echo service fails on any dual-stack name: `getent
  # hosts` answers with the AAAA while the echo answers with the A.
  DOMAIN_IPS="$(getent ahosts "$DOMAIN" 2>/dev/null | awk '{print $1}' | sort -u || true)"
  HOST_IPS="$(ip -o addr show scope global 2>/dev/null | awk '{print $4}' | cut -d/ -f1 | sort -u || true)"
  for extra in "$(curl -sf --max-time 5 https://api.ipify.org || true)" \
               "$(curl -sf --max-time 5 https://api6.ipify.org || true)"; do
    [ -n "$extra" ] && HOST_IPS="$HOST_IPS
$extra"
  done

  RESOLVES_HERE=0
  for ip in $DOMAIN_IPS; do
    for own in $HOST_IPS; do
      [ "$ip" = "$own" ] && RESOLVES_HERE=1
    done
  done

  if [ "$RESOLVES_HERE" -eq 1 ]; then
    log_info "$DOMAIN resolves to this host, requesting a certificate"
    sudo sed -i "s/server_name [^;]*;/server_name ${DOMAIN};/" /etc/nginx/sites-available/outofband.conf
    sudo nginx -t
    sudo systemctl reload nginx
    sudo certbot --nginx -d "$DOMAIN" --redirect --agree-tos -m "$EMAIL" -n
    CERT_ISSUED=1
  else
    log_warn "$DOMAIN resolves to [$(echo $DOMAIN_IPS | tr '\n' ' ')] which is not an address on this host, skipping certbot"
  fi
}

if [ "$CERT_ISSUED" -eq 1 ]; then
  log_info "install complete, serving https://$DOMAIN"
else
  log_info "install complete, serving http://$DOMAIN without a certificate"
  log_info "  once $DOMAIN points at this host: sudo certbot --nginx -d $DOMAIN --redirect --agree-tos -m $EMAIL -n"
fi
