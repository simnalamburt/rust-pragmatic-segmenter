rust-pragmatic-segmenter [![version]][crates.io]
========
Rust port of [pySBD] v3.1.0 and Ruby [pragmatic_segmenter]. **[Documentations]**

### How to build
```bash
sudo apt install -y libclang-dev
cargo build

# Run benchmark
cargo run --release --example benchmark
```

### TODOs
- [ ] Perfectly match the behavior with pySBD (current: 99%)
- [ ] Support languages other than English
- [ ] Remove regexes with look around and back references
- [ ] Try Intel Hyperscan
- [ ] Fix mistakes of pySBD, possibly send patches to the upstream
- [ ] Optimize copies and allocations
- [ ] Use proper error types instead of Boxed error
- [ ] Import test cases from pySBD and ruby pragmatic_segmenter

&nbsp;

--------
*rust-pragmatic-segmenter* is primarily distributed under the terms of both the
[Apache License (Version 2.0)] and the [MIT license]. See [COPYRIGHT] for
details.

[version]: https://badgen.net/crates/v/pragmatic-segmenter
[crates.io]: https://crates.io/crates/pragmatic-segmenter
[Documentations]: https://docs.rs/pragmatic-segmenter

[pySBD]: https://github.com/nipunsadvilkar/pySBD
[pragmatic_segmenter]: https://github.com/diasks2/pragmatic_segmenter

[MIT license]: LICENSE-MIT
[Apache License (Version 2.0)]: LICENSE-APACHE
[COPYRIGHT]: COPYRIGHT
