use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::process::exit;

extern crate fnv;
use fnv::FnvHashMap;

extern crate rayon;
use rayon::prelude::*;

/// Attempt to open a file, read it, and parse it into a vec of Strings
pub fn file_to_lines<P>(filename: P) -> Vec<String>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).expect("Could not find file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|line| line.expect("Could not parse line"))
        .collect()
}

/// Generate a pattern of integers from a string of ASCII characters
// "AB" generates a pattern of 01
// "CD" generates a pattern of 01
// "ABAB" generates a pattern of 0101
// "CDCD" generates a pattern of 0101
#[inline]
pub fn generate_pattern(haystack: &str) -> Vec<u8> {
    // neither stack nor pattern will need to re-allocate
    let mut total = 0u8;
    // ASCII uppercase is decimal 65 - 90
    // We could cope with extended ASCII by using 255
    let mut stack = [0u8; 128];
    let mut pattern = Vec::with_capacity(haystack.len());
    // it's safe to use bytes here, since ASCII is one byte per character
    for &byte in haystack.as_bytes() {
        // casting u8 to usize casts from the byte to 0…127
        // if needle has a "seen" value of 0:
        // the total is bumped by 1, ensuring each new byte gets a higher number
        // the new total is assigned to the stack at the byte position
        // needle is set to total
        // the ("seen" value - 1) is pushed onto the pattern
        if let Some(needle) = stack.get_mut(byte as usize) {
            if *needle == 0 {
                total += 1;
                *needle = total;
            }
            pattern.push(*needle - 1)
        } else {
            println!("Got a non-uppercase ASCII character.");
            exit(1)
        }
    }
    pattern
}

/// Perform a frequency count of integer sequences
#[inline]
pub fn count_frequency(patterns: &[Vec<u8>]) -> u32 {
    // Vec<u8> is hashable, so we can use a HashMap to carry out a frequency count
    // The Fowler-Noll-Vo hashing function is faster when hashing integer keys
    // resistance to DoS attacks isn't a priority here
    let mut frequency: FnvHashMap<&[u8], u32> =
        FnvHashMap::with_capacity_and_hasher(patterns.len(), Default::default());
    patterns
        .iter()
        // build up a frequency count of all patterns
        .for_each(|pattern| *frequency.entry(pattern).or_insert(0) += 1);
    // retain value counts greater than 1, and sum them
    frequency
        .par_iter()
        .filter(|&(_, &v)| v > 1) // retain frequencies > 1
        .fold(|| 0, |acc, entry| acc + entry.1) // retain only values
        .sum() // total frequencies > 1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_count() {
        let strings = vec![
            "LALALA", "XOXOXO", "GCGCGCÜ", "HHHCCC", "BBBMMM", "EGONUH", "HHRGOE"
        ];
        let patterns: Vec<_> = strings
            .iter()
            .map(|string| generate_pattern(string))
            .collect();
        let counts = count_frequency(&patterns);
        assert_eq!(counts, 5);
    }
}
