use onig::{Error, Regex, RegexOptions, Syntax};

pub struct Rule(Regex, &'static str);

impl Rule {
    pub fn new(regex: &str, replace: &'static str) -> Result<Self, Error> {
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
