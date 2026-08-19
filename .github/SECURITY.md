# Security Policy

Last updated: 2026-08-19

Thank you for helping keep this project secure. This document explains how to responsibly report security vulnerabilities and how we handle reports.

## Supported Versions
We only provide security fixes for the latest stable release(s). If you are unsure which versions are supported, open a confidential report and we'll confirm.

## How to report a vulnerability (preferred)
Please report security issues privately so we can investigate and coordinate a fix before public disclosure.

Preferred options:
- Use GitHub Security Advisories (recommended): https://github.com/Yotsawarit/bnn-code/security/advisories (if enabled)
- Email: security@yotsawarit.dev (replace with your contact)
- If you prefer PGP-encrypted mail, use this key: <PGP KEY ID or URL>

When reporting, please include:
- A short summary of the issue.
- A step-by-step reproduction (preferably minimal code or PoC).
- Impact assessment (what an attacker can do).
- Test environment and versions.
- Any suggested fixes or patches (optional).
- Whether you want to be publicly acknowledged.

Report template (copy into your message):
- Title:
- Affected component(s)/versions:
- CVE requested? (yes/no)
- Reproduction steps / proof-of-concept:
- Impact assessment:
- Contact info / PGP key (if requesting encryption):

## What to expect
- Acknowledgement: within 3 business days.
- Initial triage: within 7 business days.
- Fix timeline: depends on severity and complexity; we aim to release a fix or mitigation within 30 days for high/critical issues when feasible.
- Coordinated disclosure: we will coordinate a disclosure timeline with the reporter. Default coordinated disclosure window is 90 days from initial report unless agreed otherwise.

## Vulnerability handling
- We will validate, triage, and prioritize issues based on impact, exploitability, and the attack surface.
- For supply-chain advisories (cargo/rustsec), we will publish advisories where appropriate and update dependencies or provide mitigations.

## Public disclosure
We ask reporters to coordinate public disclosure with us. If the reporter discloses a vulnerability without coordination, we reserve the right to respond publicly as necessary.

## Safe harbor
If you are a good-faith security researcher who follows this policy and acts lawfully, we will not pursue legal action. Do not access or destroy data you are not authorized to access. Avoid denial-of-service, data exfiltration, or other intrusive testing on production systems.

## Acknowledgements
We appreciate responsible disclosures. If you request recognition, we will list you in an acknowledgements section unless you request anonymity.

## Out-of-scope
- Issues in third-party services not controlled by this repo (report to the vendor).
- Low-signal, non-actionable items without reproduction steps.
- Reports that involve social engineering or accidental leaks of private data outside the project.

## Contact / Security team
- Primary contact: security@yotsawarit.dev
- GitHub: https://github.com/Yotsawarit

If you prefer, you can open a private GitHub Security Advisory for this repository (recommended).
