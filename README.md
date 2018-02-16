# Patterns
You'll need a recent Rust installation:
Go to https://www.rustup.rs, and follow the instructions.  

If you're on Windows, see [here](https://github.com/rust-lang-nursery/rustup.rs/#other-installation-methods).

Once Rust is installed:
- clone this repo, if you don't have it on your local machine
- open a terminal, and enter the repo directory
- run `CARGO_INCREMENTAL="0" cargo build --release`
- run `target/release/patterns words.txt`
    - if you'd prefer to use a different input corpus, specify its full path.

The number you see printed out is the number of "friendly" strings, i.e. those that have at least one matching pattern, according to the rules at https://mimi.io/en/challenge/

The `sha-256` hash of the text input file (`words.txt`, it's included in this repo) is:  
`aed3d37e660fe1714ccc42185ec5a0d0a3b6f17694e765a37e97fe93ee21717e`

# Benchmarks
The binary was compiled with link-time-optimisation.  
On a 3.4 GHz Core i7, the optimised version runs in **505 ms**.  

Optimisation details:
Wherever possible, operations are parallelised using the [Rayon](https://github.com/rayon-rs/rayon) library, and instead of the standard hash function, a hashing function based on the [Fowler-Noll-Vo](https://github.com/servo/rust-fnv) function is used. This is considerably faster than the default SipHash function for small integer keys, but is far less resistant to DoS attacks.  

The unoptimised version, which uses no parallelism and the standard hash function, runs in **1457 ms**.

# Licence
The MIT license. See [licence.txt](licence.txt) for details.

# Copyright
Stephan Hügel, 2018.
