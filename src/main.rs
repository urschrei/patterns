// compile using CARGO_INCREMENTAL="0" cargo build --release
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

extern crate rayon;
use rayon::prelude::*;

extern crate fnv;
use fnv::FnvHashMap;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

/// Attempt to open a file, read it, and parse it into a vec of Strings
fn file_to_lines<P>(filename: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Could not find file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|line| line.expect("Could not parse line"))
        .collect()
}

/// Generate patterns of integers from uppercase ASCII strings
// We do this by creating a "stack" of characters that we have seen:
// Each time we encounter a new byte, we push it onto the stack
// and increment our pattern with its 0-indexed position in the stack.
// "AB" generates a pattern of 01
// "CD" generates a pattern of 01
// "ABAB" generates a pattern of 0101
// "CDCD" generates a pattern of 0101
fn generate_pattern(haystack: &str) -> Vec<u8> {
    // neither stack nor pattern will need to re-allocate
    let mut stack: Vec<&u8> = Vec::with_capacity(haystack.len());
    let mut pattern = Vec::with_capacity(haystack.len());
    for byte in haystack.as_bytes() {
        // it's safe to use bytes here, since ASCII is one byte per character
        // if a match is found: push the index at which it was found onto the pattern
        // otherwise, push a new entry for that byte onto the stack,
        // then push its index onto the pattern.
        // u8 is big enough to cover all of ASCII
        if let Some(needle) = stack.iter().position(|&elem| elem == byte) {
            pattern.push(needle as u8)
        } else {
            stack.push(byte);
            pattern.push(stack.len() as u8 - 1);
        }
    }
    pattern
}

/// Perform a frequency count of integer sequences
fn count_frequency(patterns: Vec<Vec<u8>>) -> u32 {
    // Vec<u8> is hashable, so we can use a HashMap to carry out a frequency count
    // The Fowler-Noll-Vo hashing function is faster when hashing integer keys
    // and we aren't concerned with DoS attacks here
    let mut frequency: FnvHashMap<Vec<u8>, u32> =
        FnvHashMap::with_capacity_and_hasher(patterns.len(), Default::default());
    // consume the input vector, populating the HashMap
    patterns
        .into_iter()
        .for_each(|pattern| *frequency.entry(pattern).or_insert(0) += 1);
    // retain value counts greater than 1, and sum them
    frequency
        .into_par_iter()
        .filter(|&(_, v)| v > 1) // retain frequencies > 1
        .fold(|| 0, |acc, hm| acc + hm.1) // retain only values
        .sum() // total frequencies > 1
}

fn main() {
    // Generate a CLI, and get input filename to process
    let command_params = App::new("patterns")
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
    let input_file = value_t!(command_params.value_of("INPUT_STRINGS"), String).unwrap();
    let strings = file_to_lines(&input_file);
    // generate patterns for each string
    let patterns = strings
        .par_iter()
        .map(|string| generate_pattern(string))
        .collect();
    let friendly = count_frequency(patterns);
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
        let counts = count_frequency(patterns);
        assert_eq!(counts, 5);
    }
}
