mod abbreviation_replacer;
mod list_item_replacer;
mod rule;
mod util;

use std::borrow::Cow;
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

    pub fn segment<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.is_empty() {
            return Cow::Borrowed(text);
        }
        let text = self.list_item_replacer.add_line_break(text);
        let text = self.abbreviation_replacer.replace(&text);

        // TODO: 구현하기
        unimplemented!();
    }
}
