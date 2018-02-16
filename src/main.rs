// compile using CARGO_INCREMENTAL="0" cargo build --release
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

extern crate rayon;
use rayon::prelude::*;

extern crate fnv;
use fnv::FnvHashMap;

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

/// Generate patterns of 8-bit integers from uppercase ASCII strings
// We do this by creating a "stack" of characters that we have seen:
// Each time we encounter a new character, we push it onto the stack
// and increment our pattern with its 0-indexed position in the stack.
// "AB" generates a pattern of 01
// "CD" generates a pattern of 01
// "ABAB" generates a pattern of 0101
// "CDCD" generates a pattern of 0101
fn generate_pattern(haystack: &str) -> Vec<u8> {
    // we assume that no strings will be longer than 256 chars
    let mut stack = String::with_capacity(256).to_owned();
    let mut pattern = vec![];
    for character in haystack.chars() {
        // it's safe to use find here, since ASCII is one byte per character
        let needle = stack.find(character);

        // if a match is found: push the index at which it was found onto the pattern
        // otherwise, push a new entry for that string onto the stack,
        // and push the index onto the pattern.
        // u8 is plenty, since there are only 26 letters in ASCII uppercase
        // it's still enough if we include 0-9, a-z, and punctuation
        match needle {
            Some(m) => pattern.push(m as u8),
            None => {
                stack.push_str(&character.to_string());
                pattern.push((stack.len() - 1) as u8)
            }
        }
    }
    pattern
}

/// Perform a frequency count of 8-bit integer sequences
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
        .filter(|&(_, v)| v > 1) // we're only interested in frequencies > 1
        .collect::<FnvHashMap<_, _>>()
        .values() // throw away the keys
        .sum() // sum the values (frequency counts)
}

fn main() {
    let strings = file_to_lines("words.txt");
    // generate patterns for each string
    let patterns: Vec<_> = strings
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
