# `cargo-remark` [![Build Status]][actions] [![Latest Version]][crates.io]

[Build Status]: https://github.com/kobzol/cargo-remark/actions/workflows/check.yml/badge.svg
[actions]: https://github.com/kobzol/cargo-remark/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/cargo-remark.svg
[crates.io]: https://crates.io/crates/cargo-remark

**Cargo subcommand that makes it possible to view LLVM [optimization remarks](https://llvm.org/docs/Remarks.html)
generated during the compilation of your crate.**

These remarks can tell you where and why has LLVM failed to apply certain optimizations. In certain cases, you can use
this knowledge to change your code so that it optimizes better.

# Installation
```bash
$ cargo install cargo-remark
```

# Usage
Output of LLVM optimization remarks has not been stabilized yet, so you will need a nightly version of rustc.
```bash
$ rustup update nightly
```
