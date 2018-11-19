use std::fs;
use std::path::Path;
use fnv::FnvHashMap;

use rayon::prelude::*;

/// Attempt to open a file, read it, and parse it into a vec of patterns
pub fn file_to_patterns<P>(filename: P) -> Vec<Vec<u8>>
where
    P: AsRef<Path>,
{
    // no need to use a BufReader since we want the entire file
    let s = fs::read_to_string(filename).expect("Couldn't read from file");
    s.par_lines().map(|line| generate_pattern(line)).collect()
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
        assert!(byte as usize <= 127, "Got a non-uppercase ASCII character!");
        // casting u8 to usize casts from the byte to 0…127
        // if needle has a "seen" value of 0:
        // the total is bumped by 1, so each new byte gets a higher number
        // the new total is assigned to the stack at the byte position
        // needle is set to total
        // the ("seen" value - 1) is pushed onto the pattern
        let mut needle = stack[byte as usize];
        if needle == 0 {
            total += 1;
            stack[byte as usize] = total;
            needle = total;
        }
        pattern.push(needle - 1)
    }
    pattern
}

/// Perform a frequency count of integer sequences
#[inline]
pub fn count_frequency(patterns: &[Vec<u8>]) -> u32 {
    // Vec<u8> is hashable
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
            "LALALA", "XOXOXO", "GCGCGC", "HHHCCC", "BBBMMM", "EGONUH", "HHRGOE",
        ];
        let patterns: Vec<_> = strings
            .iter()
            .map(|string| generate_pattern(string))
            .collect();
        let counts = count_frequency(&patterns);
        assert_eq!(counts, 5);
    }
}
