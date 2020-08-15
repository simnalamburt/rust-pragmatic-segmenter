use onig::{Regex, RegexOptions, Syntax};

use crate::SegmenterResult;

#[must_use]
pub fn re(regex: &str) -> SegmenterResult<Regex> {
    Ok(Regex::with_options(
        regex,
        RegexOptions::REGEX_OPTION_NONE,
        Syntax::ruby(),
    )?)
}

#[must_use]
pub fn re_i(regex: &str) -> SegmenterResult<Regex> {
    Ok(Regex::with_options(
        regex,
        RegexOptions::REGEX_OPTION_IGNORECASE,
        Syntax::ruby(),
    )?)
}
