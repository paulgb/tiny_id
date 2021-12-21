use tiny_id::ShortCodeGenerator;

const USAGE_MESSAGE: &str = "Usage: cargo run --example generate -- [alphabet size] [id length] [number of ids to generate]";
const FULL_ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn main() {
    #[cfg(not(feature = "getrandom"))]
    panic!("Generate can only be used with the crate feature \"rand\".");

    #[cfg(feature = "getrandom")]
    {
        let mut args = std::env::args();
        args.next();

        let alphabet_size: u16 = args
            .next()
            .expect(USAGE_MESSAGE)
            .parse()
            .expect("Expected argument 1 (alphabet size) to be a number.");
        let id_length: usize = args
            .next()
            .expect(USAGE_MESSAGE)
            .parse()
            .expect("Expected argument 2 (id length) to be a number.");
        let num_to_generate: u16 = args
            .next()
            .expect(USAGE_MESSAGE)
            .parse()
            .expect("Expected argument 3 (number of ids to generate) to be a number.");

        let alphabet: Vec<char> = FULL_ALPHABET[..(alphabet_size as usize)].chars().collect();

        let mut generator = ShortCodeGenerator::with_alphabet(alphabet, id_length);

        for _ in 0..num_to_generate {
            println!("{}", generator.next_string());
        }
    }
}
