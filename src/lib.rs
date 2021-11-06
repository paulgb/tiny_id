#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "rand")]
use rand::{thread_rng, Rng};

/// Compute the prime factors of a given number in ascending order.
///
/// This only needs to go up to the size of our alphabet, which will usually
/// be in the range of 25-50 characters, so we don't have to do anything
/// fancy, just use a loop to keep cutting the number down to size.
fn factorize(mut n: u32) -> Vec<u32> {
    let mut result = Vec::new();
    'outer: while n > 1 {
        let last = result.last().cloned();
        for i in last.unwrap_or(2)..n {
            if n % i == 0 {
                if last != Some(i) {
                    result.push(i)
                }
                n /= i;
                continue 'outer;
            }

            if i * i > n {
                break;
            }
        }

        if result.last() != Some(&n) {
            result.push(n)
        }
        break;
    }

    result
}

fn generate_a(m_base: u32) -> u32 {
    let factors = factorize(m_base);
    let mut prod = factors.into_iter().rfold(1, |lhs, rhs| lhs * rhs);

    // LCG calls for (a - 1) to be divisible by 4 if m is divisible
    // by 4. Since we are operating on the *base* m, i.e. m = m_base ^ l
    // with l > 1, m_base being even implies that m_base is divisible by 4.
    // In these cases prod is already even, so we double it to make it
    // divisible by 4.
    if m_base % 2 == 0 {
        prod *= 2
    }

    prod + 1
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct LinearCongruentMultiplier {
    /// The last value generated by this LCM.
    last: u64,

    /// The modulus.
    m: u64,

    /// The increment.
    c: u64,

    /// The multiplier.
    a: u64,
}

impl LinearCongruentMultiplier {
    pub fn new(seed: u64, m: u64, c: u64, a: u64) -> Self {
        Self {
            last: seed,
            m,
            c,
            a,
        }
    }

    pub fn next(&mut self) -> u64 {
        self.last = (self.a * self.last + self.c) % self.m;
        self.last
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ShortCodeGenerator<T: Copy> {
    lcm: LinearCongruentMultiplier,
    offset: u64,
    alphabet: Vec<T>,
    length: u32,
}

impl ShortCodeGenerator<char> {
    #[cfg(feature = "rand")]
    pub fn new_numeric(length: usize) -> ShortCodeGenerator<char> {
        Self::with_alphabet(
            "0123456789"
                .chars()
                .collect(),
            length,
        )
    }

    #[cfg(feature = "rand")]
    pub fn new_lowercase_alphanumeric(length: usize) -> Self {
        Self::with_alphabet(
            "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect(),
            length,
        )
    }

    #[cfg(feature = "rand")]
    pub fn new_alphanumeric(length: usize) -> Self {
        Self::with_alphabet(
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .collect(),
            length,
        )
    }

    #[cfg(feature = "rand")]
    pub fn new_uppercase(length: usize) -> Self {
        Self::with_alphabet(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .collect(),
            length,
        )
    }

    pub fn next_string(&mut self) -> String {
        self.next().into_iter().collect()
    }
}


impl<T: Copy> ShortCodeGenerator<T> {
    #[cfg(feature = "rand")]
    pub fn with_alphabet(alphabet: Vec<T>, length: usize) -> Self {
        let mut rng = thread_rng();

        let m_base = alphabet.len() as u32;
        let m = (m_base as u64).pow(length as u32);
        let a = generate_a(m_base) as u64;
        let seed = rng.gen_range(0..m) as u64;
        let offset = rng.gen_range(0..m) as u64;

        Self {
            alphabet,
            lcm: LinearCongruentMultiplier::new(seed, m, 1, a),
            offset,
            length: length as u32,
        }
    }

    pub fn next_int(&mut self) -> u64 {
        let mut result = self.lcm.next();

        result = (result + self.offset) % self.lcm.m;

        result
    }

    pub fn next(&mut self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.length as usize);
        let alphabet_size = self.alphabet.len() as u64;
        let mut value = self.next_int();

        for _ in 0..self.length {
            result.push(self.alphabet[(value % alphabet_size) as usize]);
            value /= alphabet_size;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_string_generator() {
        assert_eq!(
            3,
            ShortCodeGenerator::with_alphabet("ABC".chars().collect(), 3)
                .next_string()
                .len()
        );
        assert_eq!(
            5,
            ShortCodeGenerator::with_alphabet("ABC".chars().collect(), 5)
                .next_string()
                .len()
        );
        assert_eq!(
            10,
            ShortCodeGenerator::with_alphabet("ABC".chars().collect(), 10)
                .next_string()
                .len()
        );
    }

    #[cfg(feature = "rand")]
    fn test_generator_helper(alphabet_size: u32, length: usize) {
        let alphabet: Vec<u32> = (0..alphabet_size).into_iter().collect();
        let permutations: u64 = (alphabet_size as u64).pow(length as u32);

        let mut gen = ShortCodeGenerator::with_alphabet(alphabet, length);
        let first = gen.next();
        let mut seen = HashSet::new();

        for i in 0..permutations {
            let next = gen.next();

            // Ensure we haven't seen this
            assert!(!seen.contains(&next));

            // If this is the last unique id generated, ensure that it equals the
            // seed.
            if i == permutations - 1 {
                assert_eq!(first, next);
            }

            seen.insert(next);
        }

        // Ensure that we've seen every possible value. This is a test-of-a-test,
        // because it should never fail (even with a faulty generator) if our other
        // asserts pass, by the pigeonhole principle.
        assert_eq!(permutations, seen.len() as u64);
    }

    #[test]
    fn test_generator() {
        test_generator_helper(3, 3);
        test_generator_helper(7, 3);
        test_generator_helper(4, 9);
        test_generator_helper(26, 4);
        test_generator_helper(44, 3);
        test_generator_helper(7, 5);
    }

    #[test]
    fn test_lcm() {
        // Examples from wikipedia:
        // https://en.wikipedia.org/wiki/Linear_congruential_generator#/media/File:Linear_congruential_generator_visualisation.svg

        {
            let mut lcm = LinearCongruentMultiplier::new(1, 9, 0, 2);

            assert_eq!(2, lcm.next());
            assert_eq!(4, lcm.next());
            assert_eq!(8, lcm.next());
            assert_eq!(7, lcm.next());
            assert_eq!(5, lcm.next());
            assert_eq!(1, lcm.next());
            assert_eq!(2, lcm.next());
        }

        {
            let mut lcm = LinearCongruentMultiplier::new(3, 9, 0, 2);

            assert_eq!(6, lcm.next());
            assert_eq!(3, lcm.next());
            assert_eq!(6, lcm.next());
        }

        {
            let mut lcm = LinearCongruentMultiplier::new(0, 9, 1, 4);

            assert_eq!(1, lcm.next());
            assert_eq!(5, lcm.next());
            assert_eq!(3, lcm.next());
            assert_eq!(4, lcm.next());
            assert_eq!(8, lcm.next());
            assert_eq!(6, lcm.next());
            assert_eq!(7, lcm.next());
            assert_eq!(2, lcm.next());
            assert_eq!(0, lcm.next());
            assert_eq!(1, lcm.next());
        }
    }

    #[test]
    fn test_factorize() {
        assert_eq!(vec![2], factorize(2));
        assert_eq!(vec![3], factorize(3));
        assert_eq!(vec![2], factorize(4));

        assert_eq!(vec![2, 5], factorize(10));
        assert_eq!(vec![3, 5], factorize(15));

        assert_eq!(vec![3], factorize(27));
        assert_eq!(vec![3, 37], factorize(111));

        assert_eq!(vec![269], factorize(269));
    }

    #[test]
    fn test_generate_a() {
        assert_eq!(13, generate_a(6));

        // Not factorizable.
        assert_eq!(8, generate_a(7));

        // Not even.
        assert_eq!(34, generate_a(99));

        // Even.
        assert_eq!(53, generate_a(26));
    }
}