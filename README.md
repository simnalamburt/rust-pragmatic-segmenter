rust-pragmatic-segmenter
========
Rust port of [pySBD] v3.1.0.

### How to build
```bash
sudo apt install -y libclang-dev
cargo build

# Run benchmark
cargo run --release --example benchmark
```

### TODOs
- [ ] Intel Hyperscan 도입한 뒤 성능 개선 벤치마크하기
- [ ] list_item_replacer에 pySBD의 구현 실수나, pySBD가 루비 버전 pragmatic
      segmenter와 다르게 구현한 부분이 다수 발견됨. pySBD와 동작을 맞춘 뒤,
      diverge 할 때 고치기
- [ ] 불필요하게 정규표현식이 많이 사용된다. 정규표현식과 문자열 복사가 필요
      없는 버전으로 바꿀 수 있으나, 그러려면 알고리즘에 많이 손대야한다. 일단은
      pySBD와 동작을 동일하게 유지하기 위해, 정규표현식을 사용하는 버전으로
      작성한다.
- [ ] Boxed error 대신 제대로 된 에러 타입 만들기
- [ ] pySBD와 루비 구현체의 테스트케이스들 가져오기

&nbsp;

--------
*rust-pragmatic-segmenter* is primarily distributed under the terms of both the
[Apache License (Version 2.0)] and the [MIT license]. See [COPYRIGHT] for
details.

[pySBD]: https://github.com/nipunsadvilkar/pySBD

[MIT license]: LICENSE-MIT
[Apache License (Version 2.0)]: LICENSE-APACHE
[COPYRIGHT]: COPYRIGHT
