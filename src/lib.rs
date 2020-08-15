// TODO: Intel Hyperscan 도입한 뒤 성능 개선 벤치마크하기
//
// TODO: list_item_replacer에 pySBD의 구현 실수나, pySBD가 루비 버전 pragmatic segmenter와 다르게
// 구현한 부분이 다수 발견됨. pySBD와 동작을 맞춘 뒤, diverge 할 때 고치기
//
// TODO: 불필요하게 정규표현식이 많이 사용된다. 정규표현식과 문자열 복사가 필요 없는 버전으로 바꿀
// 수 있으나, 그러려면 알고리즘에 많이 손대야한다. 일단은 pySBD와 동작을 동일하게 유지하기 위해,
// 정규표현식을 사용하는 버전으로 작성한다.
//
// TODO: Boxed error 대신 제대로 된 에러 타입 만들기
//
// TODO: pySBD와 루비 구현체의 테스트케이스들 가져오기

mod abbreviation_replacer;
mod list_item_replacer;
mod rule;
mod util;

use std::error::Error;

use abbreviation_replacer::AbbreviationReplacer;
use list_item_replacer::ListItemReplacer;

type SegmenterResult<T> = Result<T, Box<dyn Error>>;

pub struct Segmenter {
    list_item_replacer: ListItemReplacer,
    abbreviation_replacer: AbbreviationReplacer,
}

impl Segmenter {
    pub fn new() -> SegmenterResult<Self> {
        Ok(Segmenter {
            list_item_replacer: ListItemReplacer::new()?,
            abbreviation_replacer: AbbreviationReplacer::new()?,
        })
    }

    pub fn segment(&self, text: &str) -> SegmenterResult<()> {
        if text.is_empty() {
            // TODO: 구현하기
            unimplemented!();
        }
        let text = self.list_item_replacer.add_line_break(text)?;
        let _text = self.abbreviation_replacer.replace(&text);

        // TODO: 구현하기
        unimplemented!();
    }
}
