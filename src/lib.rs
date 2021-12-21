#![doc = include_str!("../README.md")]

mod lcm;

use lcm::LinearCongruentMultiplier;
use rand_chacha::ChaCha12Rng;

#[cfg(feature = "getrandom")]
use rand_chacha::rand_core::SeedableRng;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use rand::Rng;

/// Stores the state required to generate short codes, and implements short code generation.
///
/// ```
/// let mut generator = tiny_id::ShortCodeGenerator::new_lowercase_alphanumeric(5);
/// let result: String = generator.next_string();
/// assert_eq!(5, result.len());
/// ```
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ShortCodeGenerator<T: Copy> {
    lcm: LinearCongruentMultiplier,
    offset: u64,
    alphabet: Vec<T>,
    length: u32,
    exhaustion_strategy: ExhaustionStrategy,
    
    /// Random number generator used to seed future LCMs if ExhaustionStrategy is
    /// ExtendLength. For other exhaustion strategies, it is set but never used because
    /// the initial LCM is never replaced.
    rng: Option<ChaCha12Rng>,
    
    /// Skip is used to enable partitioning. It forces the generator to skip
    /// over the given number of values between generated codes, enabling
    /// other partitions to use those codes.
    skip: Option<u32>,
    
    /// When skip is in use, we do not want to skip the first value generated
    /// by an rng, so skip_after_next is initially false. When the first random
    /// value is generated, it is set to true, enabling the skip before subsequent
    /// random generations.
    #[cfg_attr(feature = "serialize", serde(default))]
    skip_before_next: bool,
}

impl ShortCodeGenerator<char> {
    /// Create a short code generator using numeric digits.
    #[cfg(feature = "getrandom")]
    pub fn new_numeric(length: usize) -> ShortCodeGenerator<char> {
        Self::with_alphabet("0123456789".chars().collect(), length)
    }

    /// Create a short code generator using lowercase alphanumeric characters.
    #[cfg(feature = "getrandom")]
    pub fn new_lowercase_alphanumeric(length: usize) -> Self {
        Self::with_alphabet(
            "0123456789abcdefghijklmnopqrstuvwxyz".chars().collect(),
            length,
        )
    }

    /// Create a short code generator using upper and lowercase alphanumeric characters.
    #[cfg(feature = "getrandom")]
    pub fn new_alphanumeric(length: usize) -> Self {
        Self::with_alphabet(
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .chars()
                .collect(),
            length,
        )
    }

    /// Create a short code generator using uppercase characters.
    #[cfg(feature = "getrandom")]
    pub fn new_uppercase(length: usize) -> Self {
        Self::with_alphabet("ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(), length)
    }

    /// Return the next short code, represented as a string.
    /// All `next_*` calls are equivalent to each other in terms of the
    /// resulting state of self.
    pub fn next_string(&mut self) -> String {
        self.next_vec().into_iter().collect()
    }
}

impl<T: Copy> ShortCodeGenerator<T> {
    pub fn into_parallel_generators(self, generators: u32) -> Vec<Self> {
        (0..generators).map(
            move |offset| {
                let mut gen = self.clone();
                if gen.skip.is_some() {
                    panic!("Can't use into_parallel_generators on a generator that is already parallel.");
                }

                for _ in 0..offset {
                    gen.next_int();
                }
                gen.skip_before_next = false;
                gen.skip = Some(generators - 1);

                gen
            }
        ).collect()
    }

    /// Create a short code generator using a given alphabet, using the given
    /// ChaCha12Rng random number generator.
    pub fn with_alphabet_and_rng(alphabet: Vec<T>, length: usize, mut rng: ChaCha12Rng) -> Self {
        use lcm::generate_a;

        let m_base = alphabet.len() as u32;
        let m = (m_base as u64).pow(length as u32);
        let a = generate_a(m_base) as u64;
        let lcm_seed = rng.gen_range(0..m) as u64;
        let offset = rng.gen_range(0..m) as u64;

        Self {
            alphabet,
            lcm: LinearCongruentMultiplier::new(lcm_seed, m, 1, a),
            offset,
            length: length as u32,
            exhaustion_strategy: ExhaustionStrategy::default(),
            rng: Some(rng),
            skip: None,
            skip_before_next: false,
        }
    }

    /// Create a short code generator using a given alphabet.
    #[cfg(feature = "getrandom")]
    pub fn with_alphabet(alphabet: Vec<T>, length: usize) -> Self {
        let mut seed: [u8; 32] = Default::default();
        getrandom::getrandom(&mut seed).expect("Error getting entropy.");
        let rng = ChaCha12Rng::from_seed(seed);
        Self::with_alphabet_and_rng(alphabet, length, rng)
    }

    fn step(&mut self) -> u64 {
        if self.lcm.exhausted() {
            match self.exhaustion_strategy {
                ExhaustionStrategy::Cycle => {}
                ExhaustionStrategy::Panic => panic!("Exhausted."),
                ExhaustionStrategy::IncreaseLength => {
                    let rng = if let Some(rng) = self.rng.clone() {
                        rng
                    } else {
                        #[cfg(feature = "getrandom")]
                        {
                            let mut seed: [u8; 32] = Default::default();
                            getrandom::getrandom(&mut seed).expect("Error getting entropy.");
                            ChaCha12Rng::from_seed(seed)
                        }

                        #[cfg(not(feature = "getrandom"))]
                        panic!("Need crate feature getrandom to increase the length of a pre-0.1.4 ShortCodeGenerator. See https://github.com/paulgb/tiny_id/issues/2")
                    };

                    // These values of self are initialized by with_alphabet_and_rng, so we preserve them
                    // on the stack and overwrite them.
                    let skip = self.skip;
                    let skip_before_next = self.skip_before_next;
                    
                    *self = ShortCodeGenerator::with_alphabet_and_rng(
                        core::mem::take(&mut self.alphabet),
                        self.length as usize + 1,
                        rng,
                    );

                    self.skip = skip;
                    self.skip_before_next = skip_before_next;
                }
            }
        }
        self.lcm.next()
    }

    /// Return the next short code, represented as an integer.
    /// All `next_*` calls are equivalent to each other in terms of the
    /// resulting state of self.
    pub fn next_int(&mut self) -> u64 {
        if self.skip_before_next {
            for _ in 0..self.skip.unwrap_or_default() {
                println!("h0");
                self.step();
            }    
        } else {
            self.skip_before_next = true;
        }

        let mut result = self.step();
        result = (result + self.offset) % self.lcm.m;

        result
    }

    /// Deprecated alias for [`ShortCodeGenerator::next_vec`].
    #[deprecated(
        since = "0.1.4",
        note = "Deprecated to avoid confusion with Iterator::next. Use next_vec instead."
    )]
    pub fn next(&mut self) -> Vec<T> {
        self.next_vec()
    }

    /// Return the next short code, represented as a vector.
    /// All `next_*` calls are equivalent to each other in terms of the
    /// resulting state of self.
    pub fn next_vec(&mut self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.length as usize);
        let alphabet_size = self.alphabet.len() as u64;
        let mut value = self.next_int();

        for _ in 0..self.length {
            result.push(self.alphabet[(value % alphabet_size) as usize]);
            value /= alphabet_size;
        }

        result
    }

    /// Set the exhaustion strategy of this short code generator. Preserves
    /// other state.
    pub fn exhaustion_strategy(mut self, strategy: ExhaustionStrategy) -> Self {
        self.exhaustion_strategy = strategy;
        self
    }
}

/// Determines what happens when all codes (for a given alphabet and length) have
/// been exhausted.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ExhaustionStrategy {
    /// Repeat the sequences of short codes, starting with the first one.
    /// This guarantees a collision if codes live indefinitely, but can be useful
    /// when codes are used temporarily as it ensures that a code is never reused
    /// until *all other possible codes* have been used between usages.
    Cycle,

    /// Increase the length of the sequence, and continue. This is the default and
    /// avoids collisions.
    IncreaseLength,

    /// Panics. This is a fail-fast option
    /// for cases where you don't expect the codes to ever become exhausted, and
    /// either creating a collision or increasing the length of the code would be
    /// incorrect behavior.
    Panic,
}

impl Default for ExhaustionStrategy {
    fn default() -> Self {
        ExhaustionStrategy::IncreaseLength
    }
}

#[cfg(feature = "getrandom")]
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_parallel_generators() {
        let mut gen = ShortCodeGenerator::new_lowercase_alphanumeric(1);
        let mut par_gens = gen.clone().into_parallel_generators(7);

        for _ in 0..10000 {
            for par_gen in &mut par_gens {
                let expected = gen.next_string();
                let actual = par_gen.next_string();
                assert_eq!(expected, actual);
            }
        }
    }

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

    #[test]
    #[should_panic]
    fn test_exhaustion_panic() {
        let mut gen_repeat =
            ShortCodeGenerator::new_numeric(2).exhaustion_strategy(ExhaustionStrategy::Panic);

        for _ in 0..100 {
            let result = gen_repeat.next_vec();
            assert_eq!(2, result.len())
        }

        gen_repeat.next_vec();
    }

    #[test]
    fn test_exhaustion_increase_length() {
        let mut gen_repeat = ShortCodeGenerator::new_numeric(2);

        for _ in 0..100 {
            let result = gen_repeat.next_vec();
            assert_eq!(2, result.len())
        }

        let result = gen_repeat.next_vec();

        assert_eq!(3, result.len());
    }

    fn test_generator_helper(alphabet_size: u32, length: usize) {
        let alphabet: Vec<u32> = (0..alphabet_size).into_iter().collect();
        let permutations: u64 = (alphabet_size as u64).pow(length as u32);

        let mut gen = ShortCodeGenerator::with_alphabet(alphabet, length)
            .exhaustion_strategy(ExhaustionStrategy::Cycle);
        let first = gen.next_vec();
        let mut seen = HashSet::new();

        for i in 0..permutations {
            let next = gen.next_vec();

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

            assert!(!lcm.exhausted());
            assert_eq!(1, lcm.next());
            assert!(!lcm.exhausted());
            assert_eq!(2, lcm.next());
            assert!(!lcm.exhausted());
            assert_eq!(4, lcm.next());
            assert!(!lcm.exhausted());
            assert_eq!(8, lcm.next());
            assert!(!lcm.exhausted());
            assert_eq!(7, lcm.next());
            assert!(!lcm.exhausted());
            assert_eq!(5, lcm.next());
            assert!(lcm.exhausted());
            assert_eq!(1, lcm.next());
            assert!(lcm.exhausted());
        }

        {
            let mut lcm = LinearCongruentMultiplier::new(3, 9, 0, 2);

            assert_eq!(3, lcm.next());
            assert_eq!(6, lcm.next());
            assert_eq!(3, lcm.next());
        }

        {
            let mut lcm = LinearCongruentMultiplier::new(0, 9, 1, 4);

            assert_eq!(0, lcm.next());
            assert_eq!(1, lcm.next());
            assert_eq!(5, lcm.next());
            assert_eq!(3, lcm.next());
            assert_eq!(4, lcm.next());
            assert_eq!(8, lcm.next());
            assert_eq!(6, lcm.next());
            assert_eq!(7, lcm.next());
            assert_eq!(2, lcm.next());
            assert_eq!(0, lcm.next());
        }
    }

    #[test]
    fn test_0_1_3_stability() {
        let mut gen: ShortCodeGenerator<char> = serde_json::from_str(r#"
        {
            "lcm": {
                "first": 715,
                "next": 715,
                "m": 3125,
                "c": 1,
                "a": 6,
                "exhausted": false
            },
            "offset": 1097,
            "alphabet": ["a", "b", "c", "d"],
            "length": 5,
            "exhaustion_strategy": "Cycle"
        }
        "#).unwrap();

        for _ in 0..100 {
            gen.next_int();
        }

        assert_eq!("cddbc", gen.next_string());

        for _ in 0..1000 {
            gen.next_int();
        }

        assert_eq!("ccadc", gen.next_string());

        for _ in 0..10000 {
            gen.next_int();
        }

        assert_eq!("adacb", gen.next_string());
    }

    #[test]
    fn test_0_1_4_stability() {
        let mut gen: ShortCodeGenerator<char> = serde_json::from_str(r#"
        {
            "lcm": {
              "first": 1,
              "next": 1,
              "m": 64,
              "c": 1,
              "a": 5,
              "exhausted": false
            },
            "offset": 16,
            "alphabet": [
              "g",
              "h",
              "i",
              "j"
            ],
            "length": 3,
            "exhaustion_strategy": "IncreaseLength",
            "rng": {
              "seed": [
                  50, 32, 156, 125, 71, 52, 54, 124, 10, 5, 100, 142, 252, 16, 120, 159, 27, 204,
                  74, 211, 3, 75, 160, 85, 87, 13, 117, 73, 214, 197, 115, 217
              ],
              "stream": 0,
              "word_pos": 6
            },
            "skip": null,
            "used": false
          }
        "#).unwrap();

        for _ in 0..100 {
            gen.next_int();
        }

        assert_eq!("ghhh", gen.next_string());

        for _ in 0..1000 {
            gen.next_int();
        }

        assert_eq!("jhgij", gen.next_string());

        for _ in 0..10000 {
            gen.next_int();
        }

        assert_eq!("jhigggg", gen.next_string());
    }
}
