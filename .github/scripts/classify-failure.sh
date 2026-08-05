#!/usr/bin/env bash
# ─── Classify a CI failure log ────────────────────────────────────────
#
# Reads a combined log file on stdin (or $1) and emits:
#
#   kind=transient|real|unknown
#   reason=<short summary>
#
# `transient` = network / package-manager / Docker pull / GitHub API
#               hiccups — worth retrying automatically.
# `real`      = compile error, test failure, missing dependency, lint
#               failure — human needs to look.
# `unknown`   = we couldn't tell. Default to `real` so we open an issue.
#
set -euo pipefail

LOG="${1:-/dev/stdin}"

if [ ! -r "${LOG}" ]; then
    echo "kind=unknown"
    echo "reason=log not readable"
    exit 0
fi

# ── Transient patterns ────────────────────────────────────────────────
TRANSIENT_PATTERNS=(
    # apt / package manager
    'Could not resolve.*debian.org'
    'Failed to fetch.*deb\.debian\.org'
    'Temporary failure resolving'
    'Hash Sum mismatch'
    'Could not get lock /var/lib/dpkg/lock'
    'Could not get lock /var/lib/apt/lists/lock'
    'Resource temporarily unavailable'
    'The repository.*is not signed'
    'GPG error.*NO_PUBKEY'

    # cargo / crates.io — be careful: do NOT match the crates.io URL hash
    # (e.g. "crates.io-1949cf8c6b5b557") which appears in path strings
    # and contains digits that look like HTTP 5xx codes.
    'error: failed to download.*crates\.io'
    'error: could not download crate'
    'warning: spurious network error'
    'Blocking waiting for file lock'
    'error: network error.*registry'
    'failed to get crates\.io index'
    'error: received 5[0-9][0-9] from crates\.io'

    # Docker
    'toomanyrequests: Rate exceeded'
    'net/http: TLS handshake timeout'
    'dial tcp: lookup.*: no such host'
    'Cannot connect to the Docker daemon'

    # GitHub API — match actual HTTP lines, not bare 5xx in hashes
    'HTTP 5[0-9][0-9]:'                # require a colon to anchor to an HTTP status line
    'API rate limit exceeded'

    # Generic network
    'Connection timed out'
    'Connection reset by peer'
    'Operation timed out'
)

# ── Real (non-transient) patterns ─────────────────────────────────────
REAL_PATTERNS=(
    'error\[E[0-9]+\]'        # rustc errors
    'error: unmatched'
    'error: expected'
    'cannot find function'
    'cargo:rustc-link-lib'
    'ld: cannot find'
    'undefined reference'
    'panicked at'
    'test failed'
    'error: test failed'
    'Could not compile'
    'error: build failed'
    'FAILED: '
    'CMake Error'
    'recipe for target.*failed'
    'fatal error:'
    'Permission denied.*Permission denied'
)

# Convert to grep -E pattern
transient_re=$(printf '|%s' "${TRANSIENT_PATTERNS[@]}")
transient_re="${transient_re:1}"
real_re=$(printf '|%s' "${REAL_PATTERNS[@]}")
real_re="${real_re:1}"

if grep -E -i -q "${transient_re}" "${LOG}"; then
    REASON=$(grep -E -i -o "${transient_re}" "${LOG}" | head -1)
    echo "kind=transient"
    echo "reason=${REASON}"
    exit 0
fi

if grep -E -i -q "${real_re}" "${LOG}"; then
    REASON=$(grep -E -i -o "${real_re}" "${LOG}" | head -1)
    echo "kind=real"
    echo "reason=${REASON}"
    exit 0
fi

echo "kind=unknown"
echo "reason=unmatched pattern (defaulting to real for safety)"
