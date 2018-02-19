// compile using CARGO_INCREMENTAL="0" cargo build --release
extern crate rayon;
use rayon::prelude::*;

mod helpers;
use helpers::{count_frequency, file_to_lines, generate_pattern};

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() {
    // Generate a CLI, and get input filename to process
    let params = App::new("patterns")
        .version(&crate_version!()[..])
        .author("Stephan Hügel <urschrei@gmail.com>")
        .about("Generate a frequency count of patterns derived from ASCII strings")
        .arg(
            Arg::with_name("INPUT_STRINGS")
                .help("A text file containing ASCII uppercase strings, one per line")
                .index(1)
                .required(true),
        )
        .get_matches();
    let input_file = value_t!(params.value_of("INPUT_STRINGS"), String).unwrap();
    let strings = file_to_lines(&input_file);
    // generate patterns for each string
    let patterns = strings
        .par_iter()
        .map(|string| generate_pattern(string))
        .collect::<Vec<Vec<u8>>>();
    let friendly = count_frequency(&patterns);
    println!("Number of friendly strings: {:?}", friendly);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_count() {
        let strings = vec![
            "LALALA", "XOXOXO", "GCGCGC", "HHHCCC", "BBBMMM", "EGONUH", "HHRGOE"
        ];
        let patterns: Vec<_> = strings
            .iter()
            .map(|string| generate_pattern(string))
            .collect();
        let counts = count_frequency(&patterns);
        assert_eq!(counts, 5);
    }
}
