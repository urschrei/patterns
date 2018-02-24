# Three Algorithm Optimizations Outside Ebbing, Missouri

Recently, I came across an ad for an interesting dev job that had a precondition: it required you to first solve a programming challenge

> Given a list of words, two "strings" are classified as "matching" if there exists a one-to-one mapping between them. Thus, the strings `FOOFOOFOO` and `BAABAABAA` are considered matching, because `F` and `B`, and `O` and `A` map to each other, producing the same "pattern".
> 
> Given a newline-delimited file of 500k strings, how many of them are "friendly"?

Setting aside for a moment the far more interesting question "What even *is* a string?", I spent a slow afternoon pondering the problem:

- The strings in the file are uppercase ASCII
- We can keep a "stack" of characters we've seen, and the order in which we've seen them
- If the first character in each string is somehow set to `0`, and subsequent new characters that are "seen" increase by 1, I can trivially compare two patterns to check whether they match.

I write a lot of Rust these days, and I was curious to see how fast it would be, but also how much more code I'd have to write compared to the usual go-to for this kind of thing: Python. By this point, all thoughts of applying for the job had long been forgotten.

## Speeding Up Your Program, 101
Before you write any Rust code, ask yourself: "Is it possible that any of the operations in my program could be carried out in a way which is embarassingly parallel?" If the answer is yes, Rayon is the easiest solution here. What do we mean by "embarrassingly parallel"? We mean
>[…] they can easily be divided into components that can be executed concurrently. ([Herlihy and Shavit, 2012](https://books.google.com/books?id=vfvPrSz7R7QC&q=embarrasingly#v=onepage&q=embarrasingly&f=false), p.14)

In this case, this is obviously true: I'm transforming strings into integer lists; each transformation is independent, and I don't need to keep track of any other state, or carry out any other operations that have side-effects. [work-stealing isn't optimal, but nbd]. In practice, this means that I'll be able to replace sequential iteration with parallel iteration in several hot loops.

The next question I asked myself is "Have I turned on [LTO](https://llvm.org/docs/LinkTimeOptimization.html#example-of-link-time-optimization)"? The drawback of LTO is that it severely extends compilation times, but I'm going to cope with that by compiling in release mode as rarely as possible. To use LTO, edit `Cargo.toml`, adding two sections:

    [profile.release]
    lto = true

    [profile.bench]
    lto = true

This will use LTO on our release and benchmark builds. The last thing I needed to do is specify how many codegen units I wanted. Recently, Rust has been improving its compile times by splitting up its codegen units, and processing them in parallel. However, this can have a performance impact. Just as with LTO, you can mitigate this at the expense of compilation time: add `codegen-units = 1` to the `release` and `bench` profiles.

## Opening a File, and Getting the Strings
This is how everyone does it. `BufReaders` are pretty fast, and easy to use.

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

I won't spend too much time on this, but if you're wondering what that `AsRef<Path>` thing is, [this](https://www.reddit.com/r/rust/comments/3ntsbn/whats_the_point_of_asref/cvr5n5f/) is a good explanation: it's a convenient way of being able to pass any of several types to the function, any of which can be used to open the file they point to. The error-handling is intentionally brutal, because it can be; if we can't open the file or map its contents into `String`s, we may as well give up.

## Generating Patterns

### The First Attempt

    fn generate_pattern(haystack: &str) -> Vec<u8> {
        let mut stack = String::with_capacity(haystack.len()).to_owned();
        let mut pattern = Vec::with_capacity(haystack.len());
        for character in haystack.chars() {
            if let Some(needle) = stack.find(character) {
                pattern.push(needle as u8)
            } else {
                stack.push_str(&character.to_string());
                pattern.push((stack.len() - 1) as u8)
            }
        }
        pattern
    }

I used an empty, mutable `String` to keep track of characters I'd "seen", and stored the pattern in a `Vec`, I pre-allocated their length to ensure that they wouldn't have to re-allocate, because that's slow. Then, I looped through the input string slice, Using `find` to check the stack for the character. `find` returns an `Option` containing the index (remember "if we could somehow set the first character to 0"), if it was found, or `None`. In that case, we push the new character onto the stack, and push its length - 1 (i.e. the new characters index position) onto the pattern. Pretty simple. Alas, the benchmark told a different tale:  
`1,288 ns/iter (+/- 990)`  
Assuming the measurement noise on my ancient laptop is constant, that's almost 1.3 ms for the string `LALALAXOXOXO`. Give me strength.

### The Second Attempt

Because the input was uppercase ASCII, I could use *bytes*. And bytes can be translated into base-10 integers very quickly. Let's see how that looks:

    fn generate_pattern(haystack: &str) -> Vec<usize> {
        let mut stack: Vec<&u8> = Vec::with_capacity(haystack.len());
        let mut pattern = Vec::with_capacity(haystack.len());
        for byte in haystack.as_bytes() {
            if let Some(needle) = stack.iter().position(|&elem| elem == byte) {
                pattern.push(needle)
            } else {
                stack.push(byte);
                pattern.push(stack.len() - 1);
            }
        }
        pattern
    }

Instead of a `String`, I was now using a `Vec` as my stack, and using the `position` method on an iterator over it to check whether I'd "seen" a byte, allowing me to avoid all the `String` overhead. What about the benchmark?  
`130 ns/iter (+/- 52)`  
An order of magnitude speedup in what is probably the hottest code in the program. This is better. But the `usize` types continued to bother me. This is all ASCII, so I should be able to use `u8` everywhere.

### The Third Attempt

In despair, I turned to [IRC](https://client00.chat.mibbit.com/?server=irc.mozilla.org&channel=%23rust). A couple of people had some interesting suggestions, and we eventually settled on:

    pub fn generate_pattern(haystack: &str) -> Vec<u8> {
        let mut total = 0u8;
        let mut stack = [0u8; 128];
        let mut pattern = Vec::with_capacity(haystack.len());
        for &byte in haystack.as_bytes() {
            if needle == 0 {
                total += 1;
                stack[byte as usize] = total;
                needle = total;
            }
            pattern.push(needle - 1)
        }
        pattern
    }

We started off with an array representing ASCII characters, all initialised to 0. If we saw a "new" byte, we bumped `total` by 1, and set that byte's entry to `total`'s current value, before pushing it onto the pattern. Otherwise, it was an existing entry, and we simply pushed its value onto the pattern. But was it faster?  
`54 ns/iter (+/- 47)`  
At this point, I was willing to move on.

## Frequency Counting
Things became slightly more complicated at this point

    pub fn count_frequency(patterns: &[Vec<u8>]) -> u32 {
        let mut frequency: HashMap<&[u8], u32> =
            HashMap::with_capacity(patterns.len());
        patterns
            .iter()
            .for_each(|pattern| *frequency.entry(pattern).or_insert(0) += 1);
        frequency
            .par_iter()
            .filter(|&(_, &v)| v > 1)
            .fold(|| 0, |acc, entry| acc + entry.1)
            .sum() // total frequencies > 1
    }

The function accepts a slice of the patterns, in case I wanted to use them for something afterwards, and then instantiates a new `HashMap` which has the same capacity as the slice, to avoid re-allocating. Next, I iterated over the slice, adding each pattern to the HashMap using its `Entry` API. This is a fast, compact way of "upserting" values: if a pattern (key) exists, bump its value by 1. Otherwise, insert it as a new key with a value of 1. In Rust versions of yore, the `for_each` method didn't exist, requiring a `for` loop.

This is also one part of the program that couldn't be trivially parallelised: because I needed mutable access to every key (I didn't know which one, if any, I'd need to update), I had to iterate sequentially – the compiler will help here in any case.

Once I'd built the `HashMap` (which is in fact a frequency table), I needed to filter, then aggregate the results:

- filter its values, retaining only counts greater than 1
- use a fold to accumulate the remaining values
- sum the result of the fold, giving me the final count.

In theory, the final step shouldn't have been necessary, because fold should accumulate the values into a single result, but Rayon's [fold](https://docs.rs/rayon/1.0.0/rayon/iter/trait.ParallelIterator.html#method.fold) is slightly different: it returns a `Struct` containing *intermediate sums* of the input sequence, which have been calculated in parallel. The number of these summed items and their sequence is non-deterministic, requiring us to specify a final `sum()`, in order to produce the count.

The benchmark showed ~15 ms. I had no idea whether that was slow, but I *did* know that Rust's default SipHash algorithm isn't the fastest, because it's also intended to be robust against DoS attacks. In this case, that's not a concern, so I swapped in the HashMap from the [`Fnv`](https://crates.io/crates/fnv) crate. The Fowler-Noll-Vo algorithm yields better hashing performance for small integer keys. And the benchmark?  
`10,001 ns/iter (+/- 500)`  

I was now ready to actually run the program. On my desktop 3.4 GHz Core i7, with a warm cache, it runs in 205 ms.

## Complexity
I was reasonably sure that the program as a whole ran in linear time: there were a few sequential passes over the input list, at most two passes over each string, a handful of hopefully constant-time HashMap operations, and a final linear-time pass over it to aggregate the result. Still, why not verify? I sliced up the input into files increasing by 5k strings each time, then ran the program on each one, timing it using `Hyperfine`. Finally, I opened a Jupyter notebook, pulled the results into a Pandas DataFrame, fitted a line using Statsmodels, and graphed the results using Matplotlib:

![stats](stats.png)

While I was using Python, I took the opportunity to write my comparison program:

    #!/usr/bin/env python
    # coding: utf-8

    from collections import Counter


    def generate_patterns(haystack):
        """ Generate tuples of integer patterns from ASCII uppercase strings """
        total = 0
        # we begin having seen no bytes
        stack = [0] * 128
        pattern = []

        for char in haystack:
            byte = ord(char)
            needle = stack[byte]
            if needle == 0:
                # a new byte. Increase the index by one
                total += 1
                # the byte is marked as 'seen' in the stack
                stack[byte] = total
                # update the result temporarily
                needle = total
            # push its 0-indexed value onto the pattern
            pattern.append(needle - 1)
        # we need tuples because lists aren't hashable
        return tuple(pattern)


    if __name__ == "__main__":
        with open("words.txt", 'r') as f:
            counts = Counter((generate_patterns(line) for line in f))
            friendly = sum(
                {pattern: count for pattern, count in counts.items() if count >
                 1}.values()
            )
        print("Number of friendly strings: %s" % friendly)

Python has several convenient features that make the rest of the program trivial to write:

- A context manager closes the file when we finish reading from it
- We can iterate over one line at a time, generating its pattern
- Generators mean we don't have to worry about intermediate allocations
- The built-in `Collections` library makes frequency-counting easy
- Dict comprehensions make filtering on values easy.

I ended up with 22 LoC, and around 7 seconds to process 500k strings. *Very* compact (Rust is around 66 LoC), but nowhere near as fast (Rust is around 33x faster). Of course, there's lots of low-hanging fruit here, and I didn't even look at NumPy, so the speed comparison isn't intended to be meaningful, but I was pleasantly surprised by the length and conciseness of my Rust program – I could probably have reached for `Itertools` and cut it down even further, but there's really no need.
