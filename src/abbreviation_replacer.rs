use std::borrow::Cow;

pub struct AbbreviationReplacer {}

impl AbbreviationReplacer {
    pub fn new() -> Self {
        AbbreviationReplacer {}
    }

    pub fn replace(&self, _text: &str) -> Cow<str> {
        // TODO: 사용하기
        let sentence_starters = [
            "A", "Being", "Did", "For", "He", "How", "However", "I", "In", "It", "Millions",
            "More", "She", "That", "The", "There", "They", "We", "What", "When", "Where", "Who",
            "Why",
        ];

        // TODO: 구현하기
        unimplemented!()
    }
}
