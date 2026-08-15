---
description: Run offline Rust verification only; never touch physical hardware
---

Run the repository's offline verification suite:

!`cargo fmt --all -- --check`
!`cargo check --workspace`
!`cargo clippy --workspace --all-targets -- -D warnings`
!`cargo test --workspace`

Do not run `cargo run`, hardware integration tests, USB tools, or any command that opens the physical keyboard.

Summarize failures with the smallest fix required.
