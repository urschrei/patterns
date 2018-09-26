// compile using CARGO_INCREMENTAL="0" cargo build --release

use patterns::{count_frequency, file_to_patterns};

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
        ).get_matches();
    let input_file = value_t!(params.value_of("INPUT_STRINGS"), String).unwrap();
    let strings = file_to_patterns(&input_file);
    // count "friendly" patterns
    let friendly = count_frequency(&strings);
    println!("Number of friendly strings: {:?}", friendly);
}
