# `tiny_id`

**Tiny ID has been superseded by [`block-id`](https://github.com/drifting-in-space/block-id)**. `block-id` has better properties for
use in a distributed setting, and is invertible. For new usages, I recommend using `block-id` over `tiny_id`.

[![GitHub Repo stars](https://img.shields.io/github/stars/paulgb/tiny_id?style=social)](https://github.com/paulgb/tiny_id)
[![wokflow state](https://github.com/paulgb/tiny_id/workflows/Rust/badge.svg)](https://github.com/paulgb/tiny-id/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/tiny-id.svg)](https://crates.io/crates/tiny-id)
[![docs.rs](https://img.shields.io/badge/docs-release-brightgreen)](https://docs.rs/tiny-id/)
[![dependency status](https://deps.rs/repo/github/paulgb/tiny_id/status.svg)](https://deps.rs/repo/github/paulgb/tiny_id)

`tiny_id` is a Rust library for generating non-sequential, *tightly-packed* short IDs.

Most other short ID generators just string together random digits. Due to the
[birthday problem](https://en.wikipedia.org/wiki/Birthday_problem), that approach
is prone to collisions. For example, a four-digit alphabetic code has a ~50% chance of
collision after 800 codes.

`tiny_id` uses a [linear congruential generator](https://en.wikipedia.org/wiki/Linear_congruential_generator)
to generate codes which do not collide while retaining only a small, constant-sized piece
of state. For the same four-digit alphabetic code, `tiny_id` has a 0% chance of collision until all 456,976 possible codes have been generated.

These codes are indended for use-cases where it's desirable to have short, human-readable
codes such that two codes generated in a row are no more likely to resemble each other than
codes that are not. It should *not* be used in cases where the codes need to be non-guessable.
They also do not guard against a [German tank problem](https://en.wikipedia.org/wiki/German_tank_problem)-type analysis by someone sufficiently motivated.

A sample of random short codes, generated by the [generate.rs](examples/generate.rs) example,
demonstrates what a sequence of these codes looks like:

```bash
paul:~/tiny_id$ target/debug/examples/generate 36 4 10
9chl
yqpv
z1rk
c18m
1tc5
2t22
ftgv
4yih
5nag
ioa0
```

## How to use it

### Basic use

```rust
use tiny_id::ShortCodeGenerator;

fn main() {
    // The length of generated codes
    let length: usize = 6;

    // Create a generator. The generator must be mutable, because each
    // code generated updates its state.
    let mut generator = ShortCodeGenerator::new_alphanumeric(length);

    // Generate the next short code, and update the internal generator state.
    let code = generator.next_string();
    assert_eq!(length, code.len());
}
```

### Alphabets

The set of characters (or arbitrary values) used to construct a short code is referred
to as an *alphabet*. `tiny_id` has several built-in alphabets, or you can provide
your own.

If you provide an alphabet rather than use one of the built-in alphabets, that
alphabet must not contain any repeated entries. This is not enforced by the library,
but failure to abide will result in collisions.

```rust
use tiny_id::ShortCodeGenerator;

fn main() {
    let length: usize = 6;

    // There are several built-in alphabets with convenience constructors.
    
    // Numeral digits (0-9), like "769458".
    ShortCodeGenerator::new_numeric(length);

    // Numeral digits and lowercase letters (a-z), like "l2sx2b".
    ShortCodeGenerator::new_lowercase_alphanumeric(length);

    // Numeral digits, lowercase, and uppercase letters, like "qAh6Gg".
    ShortCodeGenerator::new_alphanumeric(length);

    // Uppercase letters only, like "MEPQOD".
    ShortCodeGenerator::new_uppercase(length);

    // You can also provide an alphabet with any unicode characters.
    // I hope you don't use it for emoji, but you could:
    ShortCodeGenerator::with_alphabet(
        "😛🐵😎".chars().collect(),
        length
    );

    // The generator can also be used with non-char types, as long
    // as they implement Copy.
    let mut gen = ShortCodeGenerator::with_alphabet(
        vec![true, false],
        length
    );

    // next_string() is only implemented on ShortCodeGenerator<char>,
    // but gen is a ShortCodeGenerator<bool>, so we need to call
    // next_vec() instead, which returns a Vec<bool>.
    let result: Vec<bool> = gen.next_vec();
    assert_eq!(length, result.len());
}
```

### Exhaustion Strategies

Eventually, all short code generators reach a point where they run out of codes of
a given length. There are three options for what to do when this happens:

- **Increment the length**. This corresponds to `ExhaustionStrategy::IncreaseLength`,
  which is the default.
- **Cycle**. This repeats the cycle of codes from the beginning. The order of codes
  is the same in every cycle. Corresponds to `ExhaustionStrategy::Cycle`.
- **Panic**. In the spirit of [fail-fast](https://en.wikipedia.org/wiki/Fail-fast),
  this panics when all codes have been used, for cases where exhaustion is unexpected
  and assumed by the rest of the program not to happen. Corresponds to
  `ExhaustionStrategy::Panic`.

```rust
use tiny_id::{ShortCodeGenerator, ExhaustionStrategy};

fn main() {
    // Increase length (default).
    
    let mut gen = ShortCodeGenerator::new_uppercase(2);

    for _ in 0..(26*26) {
        let result = gen.next_vec();
        assert_eq!(2, result.len());
    }

    // We've exhausted all options, so our next code will be longer.
    let result = gen.next_vec();
    assert_eq!(3, result.len());

    // Cycle.
    
    let mut gen = ShortCodeGenerator::new_uppercase(2)
        .exhaustion_strategy(ExhaustionStrategy::Cycle);
    
    let first = gen.next_vec();

    for _ in 0..(26*26-1) {
        let result = gen.next_vec();
        assert_eq!(2, result.len())
    }

    // We've exhausted all options, so our next code will be a
    // repeat of the first.
    let result = gen.next_vec();
    assert_eq!(first, result);
}
```

## How it works

A [linear congruential generator](https://en.wikipedia.org/wiki/Linear_congruential_generator)
(LCG) is a simple, non-cryptographic pseudorandom number generator.

LCGs are interesting because they generate numbers in a cycle, and the length of that cycle
as a function of the parameters to the LCG is well-studied. In particular, one thing that's
well understood is how to make an LCG that generates the numbers 1..m with a cycle size of m,
i.e., to generate a permutation of the numbers 1..m. This is called the Hull-Dobell Theorem.

For an alphabet of size `N` and an ID length of `L`, there are `N ^ L` possible codes. We can
convert back and forth between the numbers `1 .. N^L` and those codes by treating the codes
as `base-N` representations of the numbers.

Combining these two facts, our approach is:
- Using the Hull-Dobell Theorem, construct an LCG such that it will “visit” every number
  from `1` to `N^L` in some random(ish) order.
- Using the base conversion method, turn each of these numbers into a short ID.

## Notes

Note that the `ShortCodeGenerator` object itself contains a small amount of
state, which is updated every time a short code is generated. Short codes must
be generated by the same `ShortCodeGenerator` object in order to avoid collisions.

With the `serde` crate option, enabled by default, `ShortCodeGenerator` objects
can be serialized to any format supported by [serde](https://serde.rs/). This
can be used to persist the state of a generator for later use. (If you are using
a custom alphabet, the type of that alphabet must also be serializable.)

The total number of possible codes (alphabet size to the power of length) must
fit in a [`u64`](https://doc.rust-lang.org/std/primitive.u64.html). If you're working
with large enough codes and alphabets that that's a problem, you probably don't need
this library anyway (as random collisions will be more rare).

Randomness is only used during the construction of `ShortCodeGenerator`.
Code generation itself is entirely deterministic based on the current generator
state.

All operations use constant time and space, except for `ShortCodeGenerator`
construction. Construction technically has time complexity superlinear to the
cardinality of the alphabet provided. For reasonable alphabet sizes (say, <1000),
this should be negligible.

## Partitioning

If you need to generate short codes from multiple generators, you can use
the built-in partitioning feature to generate multiple short code generators
that generate codes from non-overlapping partitions of the code space.

Internally, this works by creating `num_partitions` clones of the generator,
“burning” a unique number from `0..num_partitions` codes on each one, and then
setting each one to skip past `num_partitions - 1` codes for every code it
generates.

Once a `ShortCodeGenerator` is partitioned, it can't be partitioned again or
repartitioned with a different number of partitions. Once partitioned, generators
may be serialized to be sent to a different machine or stored for later.

```rust
use tiny_id::{ShortCodeGenerator, ExhaustionStrategy};

fn main() {
    // This returns a vector of two short code generators.
    let mut generators: Vec<ShortCodeGenerator<_>> = ShortCodeGenerator::new_uppercase(5)
        .into_partitioned_generators(2);

    // Codes generated by the two generators will never overlap, because
    // they come from separate code spaces.
    assert_ne!(generators[0].next_string(), generators[1].next_string());
}
```

## Alternatives

- If you just want unique identifiers and don't care about how long they are,
  use [uuid](https://github.com/uuid-rs/uuid). It has the advantage that it
  does not need any state.
- If you are already assigning unique integers (e.g. an `AUTO INCREMENT` in SQL)
  and don't care if they are sequential, you could just convert those to and from
  strings using base conversion.
