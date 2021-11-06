# `tiny_id`

`tiny_id` is a Rust library for generating non-sequential, *tightly-packed* short IDs.

Most other short ID generators just string together random digits. Due to the
[birthday problem](https://en.wikipedia.org/wiki/Birthday_problem), that approach
is prone to collisions. For example, a four-digit alphabetic code has a 50% of
collision after 800 codes.

`tiny_id` uses a [linear congruential generator](https://en.wikipedia.org/wiki/Linear_congruential_generator)
to generate codes which do not overlap while retaining only a small, constant-sized piece
of state. For the same four-digit alphabetic code, `tiny_id` has a 0% chance of collision until all 456,976 possible codes have been generated.

These codes are indended for use-cases where it's desirable to have short, human-readable
codes such that two codes generated in a row are no more likely to resemble each other than
codes that are not. It should *not* be used in cases where the codes need to be non-guessable.
They also do not guard against a [German tank problem](https://en.wikipedia.org/wiki/German_tank_problem)-type analysis by someone sufficiently motivated.

## How to use it

```rust
use tiny_id::ShortCodeGenerator;

fn main() {
    // The length of generated codes
    let length = 6usize;

    // Create a generator. The generator must be mutable, because each
    // code generated updates its state.
    let mut generator = ShortCodeGenerator::new_alphanumeric(length);

    // Generate the next short code, and update the internal generator state.
    let code = generator.next_string();
    assert_eq!(length, code.len());

    // There are several built-in alphabets with convenience constructors.
    
    // Numeral digits (0-9), like "769458".
    ShortCodeGenerator::new_numeric(length);

    // Numeral digits and lowercase letters (a-z), like "l2sx2b".
    ShortCodeGenerator::new_lowercase_alphanumeric(length);

    // Numeral digits, lowercase, and uppercase letters, like "qAh6Gg".
    ShortCodeGenerator::new_alphanumeric(length);

    // Uppercase letters only, like "MEPQOD".
    ShortCodeGenerator::new_uppercase(length);

    // You can also provide an alphabet with any unicode characters:
    ShortCodeGenerator::with_alphabet(
        "😛🐵😎".chars().collect(),
        length
    );

    // The generator can also be used with non-char types, as long
    // as they are Copy.
    let mut gen = ShortCodeGenerator::with_alphabet(
        vec![true, false],
        length
    );

    // `next_string()` is only implemented on ShortCodeGenerator<char>,
    // but we are using a ShortCodeGenerator<bool>, so we need to call
    // `next()` instead, which returns a `Vec<bool>`.
    let result: Vec<bool> = gen.next();
    assert_eq!(length, result.len());
}
```