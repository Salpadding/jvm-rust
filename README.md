# jvm-rust
a toy jvm in rust

1. require rust nightly
2. require openjdk8/jre/lib/rt.jar

## Build

```sh
cargo build --release
```

## Test

```sh
export CLASSPATH=.:test/rt.jar # set classpath, provide your rt.jar
./target/relase/jvm-rust test/FibonacciTest
```