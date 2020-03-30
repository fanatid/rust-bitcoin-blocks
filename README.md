# rust-bitcoin-blocks

Another project for learning [Rust](https://www.rust-lang.org/).

In another project where I explore Rust I need analyze bitcoin blocks from [bitcoind](https://github.com/bitcoin/bitcoin/). As easiest way I start used JSON encoded blocks and deserialization them with [serde](https://serde.rs/), but in same time it was interested for me how faster will be parse hex encoded blocks (and what will be faster: Rust, Node.js or C++ code from original bitcoin daemon?).

## Compiling and benchmarks

- C++

```
# In `bitcoin` submodule we need create config file
./configure --disable-wallet --disable-tests --disable-bench --disable-zmq --without-utils --without-daemon --without-gui --with-libs

# Benchmark itself in `bench-cpp.cpp`, but we also need some other files for compiling
g++ -o bench-cpp \
    -I./bitcoin/src \
    -DHAVE_CONFIG_H \
    -DBUILD_BITCOIN_INTERNAL \
    -O3 \
    bench-cpp.cpp \
    bitcoin/src/primitives/transaction.cpp \
    bitcoin/src/uint256.cpp \
    bitcoin/src/util/strencodings.cpp \
    bitcoin/src/script/script.cpp \
    bitcoin/src/crypto/sha256.cpp \
    bitcoin/src/crypto/sha256_sse4.cpp

# Run benchmark
./bench-cpp
```

- Rust

```
# Compiling and run benchmark
cargo run --release

# It's also possible run with `bench`
cargo bench
```

- Node.js

```
./bench-node.js
```

## Results

(my numbers)

| | C++ | Node.js | Rust |
|:-:|:-:|:-:|:-:|
| HEX | 11.88ms | 9.96ms | 10.64ms |
| JSON | - | 22.29ms | 59.45ms |

## Conclusions?

It's was obvious that Node.js will be fastest for JSON, but it I was surprised that it's fast for hex too! From other side, C++ and Rust include deallocation for created structures, while Node.js probably not and will clear heap from not used objects later. In general it's does not matter what use for parsing blocks from hex (or bytes), C++/Node.js/Rust nearly equal for this task.
