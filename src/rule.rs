use crate::SegmenterResult;
use onig::{Regex, RegexOptions, Syntax};

pub struct Rule(Regex, &'static str);

impl Rule {
    #[must_use]
    pub fn new(regex: &str, replace: &'static str) -> SegmenterResult<Self> {
        Ok(Rule(
            Regex::with_options(regex, RegexOptions::REGEX_OPTION_NONE, Syntax::ruby())?,
            replace,
        ))
    }

    #[must_use]
    pub fn replace_all(&self, text: &str) -> String {
        self.0.replace_all(text, self.1)
    }
}
