// TODO: lookaround 필요없는 regex들은
// Rust regex crate나 Intel hyperscan으로 바꾸면 성능 올라감
//
// TODO: list_item_replacer에 pySBD의 구현 실수나, pySBD가 루비 버전 pragmatic segmenter와 다르게
// 구현한 부분이 다수 발견됨. pySBD와 동작을 맞춘 뒤, diverge 할 때 고치기
//
// TODO: 불필요하게 정규표현식이 많이 사용된다. 정규표현식과 문자열 복사가 필요 없는 버전으로 바꿀
// 수 있으나, 그러려면 알고리즘에 많이 손대야한다. 일단은 pySBD와 동작을 동일하게 유지하기 위해,
// 정규표현식을 사용하는 버전으로 작성한다.

mod rule;
mod list_item_replacer;

use std::error::Error;

use list_item_replacer::ListItemReplacer;

// TODO: 에러 핸들링 바르게 하기, boxed error 안쓰기
type SegmenterResult<T> = Result<T, Box<dyn Error>>;

pub struct Segmenter {
    list_item_replacer: ListItemReplacer,
}

impl Segmenter {
    pub fn new() -> SegmenterResult<Self> {
        Ok(Segmenter {
            list_item_replacer: ListItemReplacer::new()?,
        })
    }

    pub fn segment(&self, text: &str) {
        let _text = self.list_item_replacer.add_line_break(text);

        // TODO: 구현하기
        unimplemented!();
    }
}
