# md5-rust
A very naive md5 implementation following [rfc1321](https://tools.ietf.org/html/rfc1321)

# Usage

Can be used as a standalone executable (`cargo run`) or as a library:

```rust
use md5;

fn main() {
    println!("{}", md5::convert("Hello, MD5 Rust!")); // 493a84f0337ef40152008d065ba842d4
}
```

# Motive
To get more familiar with Rust, as well as provide a concretely defined program to make.

The code is broken into each step defined in the RFC document with my best interpretation
regarding how things are supposed work, as well as comments supporting the thinking.

Tests are a result of the RFC test cases and all pass.

---
# Disclaimer
As per [rusts own documentation](https://docs.rs/md5/0.7.0/md5/#security-warning)
>_MD5 should be considered [cryptographically broken and unsuitable for further use](https://www.kb.cert.org/vuls/id/836068)_
so don't use it, and certainly don't use this.
