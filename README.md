# doublecross

[![Build Status](https://travis-ci.org/oefd/doublecross.svg?branch=master)](https://travis-ci.org/oefd/doublecross)
[![License](https://img.shields.io/crates/l/doublecross.svg)](https://github.com/oefd/doublecross)
[![Cargo](https://img.shields.io/crates/v/doublecross.svg)](https://crates.io/crates/doublcross)
[![Documentation](https://docs.rs/doublecross/badge.svg)](https://docs.rs/doublecross)

Bi-directional channels for communicating between threads based on [crossbeam-channel](https://docs.rs/crossbeam-channel/0.2.6/crossbeam_channel/).

# example

```rust
let (left, right) = unbounded::<bool, i32>();
thread::spawn(move || {
    loop {
        let val = right.recv().unwrap();
        right.send(val % 2 == 0);
    }
});

for i in 0..10 {
    left.send(i);
    if left.recv().unwrap() {
        println!("{} is even", i);
    }
}
```

# usage

```toml
[dependencies]
doublecross = "0.2"
```

# license

Licensed under the terms of of the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/).
