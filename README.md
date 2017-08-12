# LR1 parser in Rust!


### Memory profiling

```sh
cargo build --release
valgrind --tool=massif ./target/release/lr1... parse_test
# XXXXX means the id that valgrind will output
ms_print massif.out.XXXXX | less


```

