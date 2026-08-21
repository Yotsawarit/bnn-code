# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Do not open a public issue for security reports.**

Use one of these private channels:

1. **GitHub Security Advisory (preferred)**: Go to `Security` tab → `Report a vulnerability` → fill **Request CVE** if needed. Attach PoC privately.
2. **Email**: yotsawarit@bnn-code (or via GitHub Sponsors contact)

We aim to respond within 48 hours and coordinate disclosure.

## Security Bounty

Found a valid RUSTSEC in our dependencies that we fix?
We pay **$50** for each valid RUSTSEC that we patch and verify with `cargo audit` / `cargo deny`.

* Hall of Fame: Yotsawarit Pudpong - `RUSTSEC-2026-0253` / `RUSTSEC-2026-0002` fix (`lru 0.18.2`)
* Report via Security tab > Private reporting.

People using PAC-CHAT (offline + secure) will see that we prioritize security.

## Past Fixes

- `lru 0.12.5 -> 0.18.2` fixes `RUSTSEC-2026-0253` (UAF in `LruCache::pop`) and `RUSTSEC-2026-0002` (Stacked Borrows `IterMut`) — verified with `cargo audit` 0 lru vuln (2 allowed: `paste`, `number_prefix` unmaintained).
- PoC kept private until fix: `poc/poc_lru.rs` (panics in Drop)

## Disclosure

Once a fix is confirmed and `cargo audit` shows 0 vuln, we publish advisory and credit the reporter (CVE/Hall of Fame).
