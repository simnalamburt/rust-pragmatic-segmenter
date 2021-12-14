use onig::{Error, Regex, RegexOptions, Syntax};

pub fn re(regex: &str) -> Result<Regex, Error> {
    Regex::with_options(
        regex,
        RegexOptions::REGEX_OPTION_NONE,
        Syntax::ruby(),
    )
}

pub fn re_i(regex: &str) -> Result<Regex, Error> {
    Regex::with_options(
        regex,
        RegexOptions::REGEX_OPTION_IGNORECASE,
        Syntax::ruby(),
    )
}
