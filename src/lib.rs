mod abbreviation_replacer;
mod list_item_replacer;
mod rule;
mod util;

use std::borrow::Cow;
use std::error::Error;

use onig::{Captures, Regex};

use abbreviation_replacer::AbbreviationReplacer;
use list_item_replacer::ListItemReplacer;
use rule::Rule;
use util::re;

type SegmenterResult<T> = Result<T, Box<dyn Error>>;

pub struct Segmenter {
    list_item_replacer: ListItemReplacer,
    abbreviation_replacer: AbbreviationReplacer,

    number_rules: Vec<Rule>,
    continuous_punctuation_regex: Regex,
    numbered_reference: Rule,
    misc_rules: Vec<Rule>,
}

impl Segmenter {
    pub fn new() -> SegmenterResult<Self> {
        Ok(Segmenter {
            list_item_replacer: ListItemReplacer::new()?,
            abbreviation_replacer: AbbreviationReplacer::new()?,

            number_rules: vec![
                // PeriodBeforeNumberRule
                // Example: https://rubular.com/r/oNyxBOqbyy
                Rule::new(r"\.(?=\d)", "∯")?,
                // NumberAfterPeriodBeforeLetterRule
                // Example: https://rubular.com/r/EMk5MpiUzt
                Rule::new(r"(?<=\d)\.(?=\S)", "∯")?,
                // NewLineNumberPeriodSpaceLetterRule
                // Example: https://rubular.com/r/rf4l1HjtjG
                Rule::new(r"(?<=\r\d)\.(?=(\s\S)|\))", "∯")?,
                // StartLineNumberPeriodRule
                // Example: https://rubular.com/r/HPa4sdc6b9
                Rule::new(r"(?<=^\d)\.(?=(\s\S)|\))", "∯")?,
                // StartLineTwoDigitNumberPeriodRule
                // Example: https://rubular.com/r/NuvWnKleFl
                Rule::new(r"(?<=^\d\d)\.(?=(\s\S)|\))", "∯")?,
            ],

            // Example: https://rubular.com/r/mQ8Es9bxtk
            continuous_punctuation_regex: re(r"(?<=\S)(!|\?){3,}(?=(\s|\Z|$))")?,

            // Example: https://rubular.com/r/UkumQaILKbkeyc
            numbered_reference: Rule::new(
                r"(?<=[^\d\s])(\.|∯)((\[(\d{1,3},?\s?-?\s?)*\b\d{1,3}\])+|((\d{1,3}\s?)?\d{1,3}))(\s)(?=[A-Z])",
                r"∯\2\r\7",
            )?,

            misc_rules: vec![
                // English.Abbreviation.WithMultiplePeriodsAndEmailRule,
                //
                // NOTE: pySBD와 루비 구현체가 다른 정규표현식을 쓴다. pySBD의 동작을 따라간다.
                //
                // Example: https://rubular.com/r/EUbZCNfgei
                Rule::new(r"([a-zA-Z0-9_])(\.)([a-zA-Z0-9_])", r"\1∮\3")?,
                // English.GeoLocationRule,
                Rule::new(r"(?<=[a-zA-z]°)\.(?=\s*\d+)", "∯")?,
                // English.FileFormatRule,
                Rule::new(
                    r"(?<=\s)\.(?=(jpe?g|png|gif|tiff?|pdf|ps|docx?|xlsx?|svg|bmp|tga|exif|odt|html?|txt|rtf|bat|sxw|xml|zip|exe|msi|blend|wmv|mp[34]|pptx?|flac|rb|cpp|cs|js)\s)",
                    "∯",
                )?,
            ],
        })
    }

    pub fn segment<'a>(&self, text: &'a str) {
        if text.is_empty() {
            // TODO
            unimplemented!()
        }

        // NOTE: 루비 버전에는 이런 처리가 없으나, pySBD 3.1.0에 이 처리가 들어갔다. pySBD와 동작을
        // 맞추기위해 동일하게 처리해준다.
        let text = text.replace('\n', "\r");

        let text = self.list_item_replacer.add_line_break(&text);

        // replace_abbreviations()
        let mut text = self.abbreviation_replacer.replace(&text);

        // replace_numbers()
        for rule in &self.number_rules {
            text = rule.replace_all(&text);
        }

        // replace_continuous_punctuation()
        let text = self
            .continuous_punctuation_regex
            .replace_all(&text, |c: &Captures| {
                let mat = c.at(0).unwrap(); // Must exists
                mat.replace('!', "&ᓴ&").replace('?', "&ᓷ&")
            });

        // replace_periods_before_numeric_references()
        //
        // Reference:
        //   https://github.com/diasks2/pragmatic_segmenter/commit/d9ec1a35
        let mut text = self.numbered_reference.replace_all(&text);

        for rule in &self.misc_rules {
            text = rule.replace_all(&text);
        }

        // split_into_segments()
        // TODO

        unimplemented!()
    }
}
