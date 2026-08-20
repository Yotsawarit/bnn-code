# Bounty Submission Draft — lru RUSTSEC-2026-0253 / RUSTSEC-2026-0002

> **Correction to BOUNTY-SUBMISSION-GUIDE.md:** the guide cited `lru 0.12.6` /
> `0.16.0` as the fix. That is **WRONG**. RUSTSEC-2026-0253 is patched in
> **lru >= 0.18.2** (jeromefroe/lru-rs#238). `0.12.6` does not exist, and
> `0.16.4` (hence `0.16.0`) is still vulnerable per `cargo audit`.

## Fix applied in bnn-code
- `Cargo.toml`: `lru = "0.16.3"` → `"0.18"` (resolves to `0.18.2`, the patched version).
- `cargo update -p lru@0.16.4 --precise 0.18.2`.
- Result: `cargo audit` clears the bnn-code **direct** `lru` advisory.
- **Remaining:** `ratatui 0.26.3` pins `lru = "^0.12.0"`, so its transitive
  `lru 0.12.5` advisory (RUSTSEC-2026-0253 / 2026-0002) remains until an
  upstream `ratatui` bump. `paste` (RUSTSEC-2024-0436) is unmaintained —
  informational only, not a vulnerability.

## Path 1 — huntr.dev
- **Repository:** `https://github.com/jeromefroe/lru-rs`
- **Version:** `0.12.5` (also affects `0.16.4`)
- **Title:** Use-after-free in `LruCache::pop()` due to lack of panic safety — RUSTSEC-2026-0253

```
## Summary
LruCache::pop() in lru < 0.18.2 is not panic-safe. If the Drop impl of a stored
value panics during pop(), internal pointers become dangling, leading to UAF
(RUSTSEC-2026-0253; related Stacked-Borrows issue RUSTSEC-2026-0002).

## Impact
Use-after-free -> potential memory corruption in any project using lru < 0.18.2,
including bnn-code (both directly and transitively via ratatui).

## Reproduction
Private reproduction in bnn-code showing panic in Drop during pop() causes
corrupted/dangling internal state when the cache is reused. See attached PoC
(poc_lru.rs, kept private).

## Fix
Update to lru >= 0.18.2, where panic safety is added (jeromefroe/lru-rs#238).
NOTE: the "0.12.6" / "0.16.0" versions cited in some write-ups are incorrect —
0.12.6 does not exist and 0.16.4 is still vulnerable. Verified in bnn-code with
`cargo update -p lru@0.16.4 --precise 0.18.2`; cargo audit shows the direct
lru advisory cleared.

## References
https://rustsec.org/advisories/RUSTSEC-2026-0253
https://rustsec.org/advisories/RUSTSEC-2026-0002
```

## Path 2 — GitHub Private Vulnerability Reporting
Same content as Path 1; select **Request CVE**.

> Note: the advisory was issued 2026-08-11 and is already fixed upstream in
> `0.18.2`, so this is a *known, patched* advisory — claim credit only if you
> contributed new analysis/PoC.
