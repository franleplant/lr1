# LR1 parser in Rust!

[![Build Status](https://travis-ci.org/franleplant/lr1.svg?branch=master)](https://travis-ci.org/franleplant/lr1)
[![](https://tokei.rs/b1/github/franleplant/lr1)](https://github.com/franleplant/lr1)

An attempt to implement a LR1 parser in Rust.
The main goal is to experiment with Rust while learning about parser construction.


### Memory profiling

```sh
cargo build --release
valgrind --tool=massif ./target/release/lr1... parse_test
# XXXXX means the id that valgrind will output
ms_print massif.out.XXXXX | less


```

