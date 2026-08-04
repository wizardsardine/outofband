#!/usr/bin/env bash
# Prints the nginx security headers snippet for a built page, with the sha256
# of every inline script substituted into the CSP.
#
# usage: render-security-headers.sh <template> <index.html>
set -euo pipefail

[ $# -eq 2 ] || { echo "usage: $(basename "$0") <template> <index.html>" >&2; exit 1; }
TEMPLATE="$1"
INDEX="$2"

[ -f "$TEMPLATE" ] || { echo "no such template: $TEMPLATE" >&2; exit 1; }
[ -f "$INDEX" ] || { echo "no such page: $INDEX" >&2; exit 1; }

HASHES="$(python3 - "$INDEX" <<'PY'
import base64, hashlib, re, sys

page = open(sys.argv[1], encoding="utf-8").read()
# Scripts with a src are covered by the host allowlist, not by a hash.
inline = re.findall(r"<script(?![^>]*\bsrc=)[^>]*>(.*?)</script>", page, re.S)
if not inline:
    sys.exit("found no inline script in %s, the page cannot be right" % sys.argv[1])
print(" ".join(
    "'sha256-%s'" % base64.b64encode(hashlib.sha256(s.encode()).digest()).decode()
    for s in inline
))
PY
)"

# Base64 never contains a pipe, so it is safe as the sed delimiter here.
sed "s|__SCRIPT_HASHES__|$HASHES|" "$TEMPLATE"
