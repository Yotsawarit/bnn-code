# Security Policy - bnn-code

## Supported Versions
| Version | Supported |
| ------- | --------- |
| 0.1.4   | ✅ |
| 0.1.3   | ✅ Hardened as of 2026-08-18 |

## Security Audit Status (2026-08-18)
This project was audited against RustSec Advisory Database.

**Result: 8/8 critical vulnerabilities PATCHED, 0 active.**

- anyhow v1.0.104 - RUSTSEC-2026-0190 [PATCHED >=1.0.103]
- chrono v0.4.45 - RUSTSEC-2020-0159 [PATCHED >=0.4.20]
- tokio v1.53.1 - RUSTSEC-2025-0023 [PATCHED >=1.44.2]
- regex v1.13.1 - RUSTSEC-2022-0013 [PATCHED >=1.5.5]
- rusqlite v0.31.0 - RUSTSEC-2021-0128 [PATCHED >=0.26.2]
- tracing v0.1.44 - RUSTSEC-2023-0078 [PATCHED >=0.1.40]
- rustls v0.23.40 - RUSTSEC-2024-0399 [PATCHED >=0.23.18]
- hyper v1.11.0 - RUSTSEC-2021-0078 [PATCHED >=0.14.10]
- dirs v5.0.1 - RUSTSEC-2020-0053 [UNMAINTAINED] -> Migrated to dirs-next in v0.1.4

Verification: `cargo audit` and `cargo deny check advisories`

## Reporting a Vulnerability
We take zero-day reports seriously.

1. **DO NOT** open a public issue.
2. Use GitHub Private Vulnerability Reporting: https://github.com/Yotsawarit/bnn-code/security/advisories/new
3. Or email: [add your security email]
4. Please include:
   - RustSec ID if known, or PoC with vulnerable version
   - `cargo tree` output
   - Suggested patch

We follow 90-day responsible disclosure.

## Reward Path
- Valid new zero-day in dependencies: We will credit you in SECURITY_AUDIT.pdf, GitHub Advisory, and release notes.
- We participate in huntr.dev and GitHub Security Lab for upstream crates.
- Critical fixes are eligible for GitHub Sponsors bonus from project owner.

## CI/CD Security
Every PR runs:
```
cargo install cargo-audit
cargo audit
```

License: MIT - Copyright (c) 2026 Mr. Yotsawarit Pudpong
