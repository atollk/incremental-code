use std::ops::Add;

const M: u64 = (1 << 61) - 1; // Mersenne prime 2^61 - 1
const B: u64 = 1_000_003;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashedString {
    length: u64,
}

impl From<&str> for HashedString {
    fn from(value: &str) -> Self {
        Self {
            length: value.len() as u64,
        }
    }
}

impl Add for HashedString {
    type Output = HashedString;

    fn add(self, rhs: Self) -> Self::Output {
        /*let shifted = mul_mod(self.hash(), pow_mod(B, rhs.len()));
        self.values.0 = add_mod(shifted, rhs.hash());
        self.values.1 += rhs.len();
        self*/
        HashedString {
            length: self.length + rhs.length,
        }
    }
}

impl HashedString {
    fn compute_hash(s: &str) -> u64 {
        let mut value = 0u64;
        for &byte in s.as_bytes() {
            value = add_mod(mul_mod(value, B), byte as u64);
        }
        value
    }

    pub fn hash(&self) -> u64 {
        self.length
    }

    pub fn len(&self) -> u64 {
        self.length
    }
}

/// Multiply two values mod M, using u128 to avoid overflow.
#[inline]
fn mul_mod(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % M as u128) as u64
}

#[inline]
fn add_mod(a: u64, b: u64) -> u64 {
    let s = a + b; // both < M < 2^61, so sum < 2^62, no u64 overflow
    if s >= M { s - M } else { s }
}

/// Fast modular exponentiation: B^exp mod M.
fn pow_mod(mut base: u64, mut exp: u64) -> u64 {
    let mut result = 1u64;
    base %= M;
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod(result, base);
        }
        base = mul_mod(base, base);
        exp >>= 1;
    }
    result
}
