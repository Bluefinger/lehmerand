#![forbid(unsafe_code)]

use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::thread;
use std::time::Instant;

const MULT: u128 = 0x12e15e35b500f16e2e714eb2b37916a5;
const MASK_LOW: u64 = 0x00000000ffffffff;
const MASK_HIGH: u64 = 0xffffffff00000000;

thread_local! {
    static RNG: Rc<Rng> = Rc::new(Rng(Cell::new({
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        thread::current().id().hash(&mut hasher);
        let hash = hasher.finish();
        (hash << 1 | 1).into()
    })));
}

/// A random number generator.
#[derive(Debug)]
pub struct Rng(Cell<u128>);

impl Rng {
    pub fn new() -> Self {
        let seed = RNG.with(|r| r.next_state());

        Rng(Cell::new(seed))
    }

    pub fn with_seed(seed: u128) -> Self {
        Rng(Cell::new((seed << 1) | 1))
    }

    #[inline]
    fn next_state(&self) -> u128 {
        let state = self.0.get();
        let state = state.wrapping_mul(MULT);
        self.0.set(state);
        state
    }

    #[inline]
    fn gen_u64(&self) -> u64 {
        (self.next_state() >> 64) as u64
    }

    pub fn u64(&self) -> u64 {
        self.gen_u64()
    }

    pub fn u32(&self) -> u32 {
        let gen = self.gen_u64();
        let low = (gen & MASK_LOW) as u32;
        let high = ((gen & MASK_HIGH) >> 32) as u32;
        
        high ^ low
    }

    pub fn u16(&self) -> u16 {
        (self.u32() >> 16) as u16
    }

    pub fn u8(&self) -> u8 {
        (self.u32() >> 24) as u8
    }

    pub fn i64(&self) -> i64 {
        let gen = self.gen_u64().to_le_bytes();

        i64::from_le_bytes(gen)
    }

    pub fn i32(&self) -> i32 {
        let gen = self.u32().to_le_bytes();

        i32::from_le_bytes(gen)
    }

    pub fn i16(&self) -> i16 {
        let gen = self.u16().to_le_bytes();

        i16::from_le_bytes(gen)
    }

    pub fn i8(&self) -> i8 {
        let gen = self.u8().to_le_bytes();

        i8::from_le_bytes(gen)
    }
}

impl Default for Rng {
    #[inline]
    fn default() -> Rng {
        Rng::new()
    }
}

impl Clone for Rng {
    fn clone(&self) -> Self {
        Rng(Cell::new(self.next_state()))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let rng = Rng::with_seed(Default::default());

        assert_ne!(rng.gen_u64(), 0);
        assert_eq!(rng.gen_u64(), 4075977849992214257);
    }

    #[test]
    fn always_unique() {
        let rng1 = Rng::new();
        let rng2 = Rng::new();

        assert_ne!(
            rng1.0.get(),
            rng2.0.get(),
            "initial states must always be unique"
        );
        assert_ne!(
            rng1.gen_u64(),
            rng2.gen_u64(),
            "unique states produce different number sequences"
        );
    }

    #[test]
    fn deterministic_clone() {
        let rng1 = Rng::with_seed(Default::default());
        rng1.gen_u64();

        let rng2 = Rng::with_seed(Default::default());
        rng2.gen_u64();

        let cloned1 = rng1.clone();
        let cloned2 = rng2.clone();

        assert_eq!(cloned1.gen_u64(), rng1.gen_u64());
        assert_eq!(cloned2.gen_u64(), rng2.gen_u64());
        assert_eq!(cloned1.gen_u64(), cloned2.gen_u64());
    }
}
