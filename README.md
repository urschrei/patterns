# Patterns
You'll need a recent Rust installation:
Go to https://www.rustup.rs, and follow the instructions.  

If you're on Windows, see [here](https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods).

Once Rust is installed:
- clone this repo, if you don't have it on your local machine
- open a terminal, and enter the repo directory
- run `CARGO_INCREMENTAL="0" cargo build --release`
- run `target/release/patterns`.

The number you see printed out is the number of "friendly" strings, i.e. those that have at least one matching pattern, according to the rules at https://mimi.io/en/challenge/

The `sha-256` hash of the text input file (`words.txt`, it's included in this repo) is:  
`aed3d37e660fe1714ccc42185ec5a0d0a3b6f17694e765a37e97fe93ee21717e`

# Licence
The MIT license. See [licence.txt](licence.txt) for details.

# Copyright
Stephan Hügel, 2018.
