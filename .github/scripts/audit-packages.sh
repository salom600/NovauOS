#!/usr/bin/env bash
# ─── Audit novau-base.list.chroot against Debian 12 bookworm ───────────
#
# Verifies that every package name in our package-lists file actually
# exists in the Debian 12 (bookworm) package indices (main, contrib,
# non-free, non-free-firmware, plus -updates and -security).
#
# Run locally before pushing ISO changes:
#   ./.github/scripts/audit-packages.sh
#
# Exits 0 if all packages are present, 1 otherwise.
#
set -euo pipefail

# Locate the list file (walk up from script dir to find repo root) — do this
# BEFORE cd-ing into the cache dir so the relative path resolves correctly.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LIST="${1:-iso-build/config/package-lists/novau-base.list.chroot}"
LIST_PATH="${REPO_ROOT}/${LIST}"

CACHE_DIR="${HOME}/.cache/novauos-pkg-audit"
mkdir -p "${CACHE_DIR}"

if [ ! -f "${LIST_PATH}" ]; then
  echo "[audit] ERROR: list file not found at ${LIST_PATH}"
  exit 1
fi

# Fetch the indices if we don't have them or they're older than 1 day.
NEED_FETCH=0
if [ ! -f "${CACHE_DIR}/ALL_NAMES.txt" ] || \
   [ "$(find "${CACHE_DIR}/ALL_NAMES.txt" -mtime +1 -print 2>/dev/null | wc -l)" -gt 0 ]; then
  NEED_FETCH=1
fi

if [ "${NEED_FETCH}" -eq 1 ]; then
  echo "[audit] Fetching Debian bookworm package indices (one-time, cached for 24h)…"
  cd "${CACHE_DIR}"
  for suite in bookworm bookworm-updates bookworm-security; do
    for area in main contrib non-free non-free-firmware; do
      for arch in amd64 all; do
        URL="https://deb.debian.org/debian/dists/${suite}/${area}/binary-${arch}/Packages.gz"
        OUT="${suite}-${area}-${arch}.Packages.gz"
        curl -sSf "$URL" -o "${OUT}" 2>/dev/null || rm -f "${OUT}"
      done
    done
  done
  zcat *.Packages.gz 2>/dev/null > ALL.txt
  grep '^Package: ' ALL.txt | awk '{print $2}' | sort -u > ALL_NAMES.txt
  cd "${REPO_ROOT}"
fi

TOTAL=$(wc -l < "${CACHE_DIR}/ALL_NAMES.txt")
echo "[audit] Index contains ${TOTAL} unique package names."

echo "[audit] Auditing ${LIST_PATH}…"
echo ""

MISSING=()
PRESENT=0
while IFS= read -r line; do
  line="${line%%#*}"
  line="$(echo "$line" | xargs)"
  [ -z "$line" ] && continue
  case "$line" in -*) continue ;; esac
  if grep -qFx "$line" "${CACHE_DIR}/ALL_NAMES.txt"; then
    PRESENT=$((PRESENT+1))
  else
    MISSING+=("$line")
  fi
done < "${LIST_PATH}"

echo "PRESENT: ${PRESENT} packages"
echo "MISSING: ${#MISSING[@]} packages"
echo ""

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "❌ The following packages are NOT available in Debian 12 bookworm:"
  echo ""
  for p in "${MISSING[@]}"; do
    # Suggest similar names
    similar=$(grep -i "^${p:0:4}" "${CACHE_DIR}/ALL_NAMES.txt" | head -3 | tr '\n' ' ')
    echo "  ✗ ${p}"
    if [ -n "${similar}" ]; then
      echo "    similar: ${similar}"
    fi
  done
  echo ""
  echo "Fix the package list at ${LIST_PATH} and re-run this audit."
  exit 1
fi

echo "✓ All packages verified."
exit 0

