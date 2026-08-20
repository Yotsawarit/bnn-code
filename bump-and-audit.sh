#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# 1. Unblock cargo: the [patch.crates-io] paste fork 404s.
#    Edit Cargo.toml to either remove the block or fix the URL, e.g.:
#    sed -i '/\[patch.crates-io\]/,/^$/d' Cargo.toml   # removes the block
#    (Already removed in this repo.)

# 2. Bump the bnn-code DIRECT lru to the patched 0.18.2.
#    RUSTSEC-2026-0253 / 2026-0002 are fixed in lru >= 0.18.2
#    (NOT 0.12.6 - that version does not exist; NOT 0.16.0 - 0.16.4 is still
#    vulnerable). Also set `lru = "0.18"` in Cargo.toml if not already.
#    NOTE: ratatui 0.26.x pins lru = "^0.12.0", so its transitive lru 0.12.5
#    advisory remains until an upstream ratatui bump - this clears the direct one.
cargo update -p lru@0.16.4 --precise 0.18.2

# 3. Audit
cargo install cargo-audit
cargo audit            # expect the bnn-code direct lru advisory gone;
                       # lru 0.12.5 (via ratatui) may remain until ratatui bumps

# 4. Commit
git add Cargo.lock
git commit -m "fix(deps): bump lru 0.16.4 -> 0.18.2 (RUSTSEC-2026-0253/0002)"
