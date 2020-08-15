use std::borrow::Cow;
use std::collections::HashSet;
use std::iter::Iterator;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, FindIter, MatchKind};
use onig::{Captures, Regex};
use unic_ucd_case::is_cased;

use crate::rule::Rule;
use crate::util::{re, re_i};
use crate::SegmenterResult;

pub struct AbbreviationReplacer {
    possessive_abbreviation_rule: Rule,
    kommanditgesellschaft_rule: Rule,
    single_letter_abbreviation_rules: Vec<Rule>,
    am_pm_rules: Vec<Rule>,

    python_splitlines_keepends: PythonSplitLines,

    abbreviations: Vec<(&'static str, Regex, Regex)>,
    prepositive_abbreviations: HashSet<&'static str>,
    number_abbreviations: HashSet<&'static str>,

    multi_period_abbreviation_regex: Regex,
    multi_period_abbreviation_replace_period: Rule,

    replace_abbreviation_as_sentence_boundary: Rule,
}

#[rustfmt::skip]
const ABBREVIATIONS: &[&str] = &[
    "adj", "adm", "adv", "al", "ala", "alta", "apr", "arc", "ariz", "ark", "art", "assn", "asst",
    "attys", "aug", "ave", "bart", "bld", "bldg", "blvd", "brig", "bros", "btw", "cal", "calif",
    "capt", "cl", "cmdr", "co", "col", "colo", "comdr", "con", "conn", "corp", "cpl", "cres", "ct",
    "d.phil", "dak", "dec", "del", "dept", "det", "dist", "dr", "dr.phil", "dr.philos", "drs",
    "e.g", "ens", "esp", "esq", "etc", "exp", "expy", "ext", "feb", "fed", "fla", "ft", "fwy",
    "fy", "ga", "gen", "gov", "hon", "hosp", "hr", "hway", "hwy", "i.e", "ia", "id", "ida", "ill",
    "inc", "ind", "ing", "insp", "is", "jan", "jr", "jul", "jun", "kan", "kans", "ken", "ky", "la",
    "lt", "ltd", "maj", "man", "mar", "mass", "may", "md", "me", "med", "messrs", "mex", "mfg",
    "mich", "min", "minn", "miss", "mlle", "mm", "mme", "mo", "mont", "mr", "mrs", "ms", "msgr",
    "mssrs", "mt", "mtn", "neb", "nebr", "nev", "no", "nos", "nov", "nr", "oct", "ok", "okla",
    "ont", "op", "ord", "ore", "p", "pa", "pd", "pde", "penn", "penna", "pfc", "ph", "ph.d", "pl",
    "plz", "pp", "prof", "pvt", "que", "rd", "rs", "ref", "rep", "reps", "res", "rev", "rt",
    "sask", "sec", "sen", "sens", "sep", "sept", "sfc", "sgt", "sr", "st", "supt", "surg", "tce",
    "tenn", "tex", "univ", "usafa", "u.s", "ut", "va", "v", "ver", "viz", "vs", "vt", "wash",
    "wis", "wisc", "wy", "wyo", "yuk", "fig",
];

const PREPOSITIVE_ABBREVIATIONS: &[&str] = &[
    "adm", "attys", "brig", "capt", "cmdr", "col", "cpl", "det", "dr", "gen", "gov", "ing", "lt",
    "maj", "mr", "mrs", "ms", "mt", "messrs", "mssrs", "prof", "ph", "rep", "reps", "rev", "sen",
    "sens", "sgt", "st", "supt", "v", "vs", "fig",
];

const NUMBER_ABBREVIATIONS: &[&str] = &["art", "ext", "no", "nos", "p", "pp"];

impl AbbreviationReplacer {
    pub fn new() -> SegmenterResult<Self> {
        Ok(AbbreviationReplacer {
            // Example: https://rubular.com/r/yqa4Rit8EY
            possessive_abbreviation_rule: Rule::new(r"\.(?='s\s)|\.(?='s$)|\.(?='s\Z)", "∯")?,

            // Example: https://rubular.com/r/NEv265G2X2
            kommanditgesellschaft_rule: Rule::new(r"(?<=Co)\.(?=\sKG)", "∯")?,

            single_letter_abbreviation_rules: vec![
                // SingleUpperCaseLetterAtStartOfLineRule
                // Example: https://rubular.com/r/e3H6kwnr6H
                Rule::new(r"(?<=^[A-Z])\.(?=\s)", "∯")?,
                // SingleUpperCaseLetterRule
                // Example: https://rubular.com/r/gitvf0YWH4
                Rule::new(r"(?<=\s[A-Z])\.(?=,?\s)", "∯")?,
            ],

            am_pm_rules: vec![
                // UpperCasePmRule
                // Example: https://rubular.com/r/Vnx3m4Spc8
                Rule::new(r"(?<= P∯M)∯(?=\s[A-Z])", ".")?,
                // UpperCaseAmRule
                // Example: https://rubular.com/r/AJMCotJVbW
                Rule::new(r"(?<=A∯M)∯(?=\s[A-Z])", ".")?,
                // LowerCasePmRule
                // Example: https://rubular.com/r/13q7SnOhgA
                Rule::new(r"(?<=p∯m)∯(?=\s[A-Z])", ".")?,
                // LowerCaseAmRule
                // Example: https://rubular.com/r/DgUDq4mLz5
                Rule::new(r"(?<=a∯m)∯(?=\s[A-Z])", ".")?,
            ],

            python_splitlines_keepends: PythonSplitLines::new(),

            abbreviations: ABBREVIATIONS
                .iter()
                .map(|&abbr| -> SegmenterResult<_> {
                    // NOTE: 여기에서도 escaped이 된 abbr을 써야하지만, pySBD와 동작을 유지하기위해
                    // 의도적으로 abbr를 바로 사용한다
                    let abbr_match = re_i(&format!(r"(?:^|\s|\r|\n){}", abbr))?;

                    // NOTE: abbr에 . 이외의 글자가 들어가게될 경우, 아래의 escape 로직도 함께
                    // 고쳐야한다.
                    let escaped = abbr.replace(r".", r"\.");
                    let next_word_start = re(&format!(r"(?<={{{}}} ).{{1}}", escaped))?;

                    Ok((abbr, abbr_match, next_word_start))
                })
                .collect::<Result<_, _>>()?,

            prepositive_abbreviations: PREPOSITIVE_ABBREVIATIONS.iter().map(|s| *s).collect(),
            number_abbreviations: NUMBER_ABBREVIATIONS.iter().map(|s| *s).collect(),

            // Example: https://rubular.com/r/xDkpFZ0EgH
            multi_period_abbreviation_regex: re_i(r"\b[a-z](?:\.[a-z])+[.]")?,

            multi_period_abbreviation_replace_period: Rule::new(r"\.", "∯")?,

            replace_abbreviation_as_sentence_boundary: Rule::new(
                r"(U∯S|U\.S|U∯K|E∯U|E\.U|U∯S∯A|U\.S\.A|I|i.v|I.V)∯((?=\sA\s)|(?=\sBeing\s)|(?=\sDid\s)|(?=\sFor\s)|(?=\sHe\s)|(?=\sHow\s)|(?=\sHowever\s)|(?=\sI\s)|(?=\sIn\s)|(?=\sIt\s)|(?=\sMillions\s)|(?=\sMore\s)|(?=\sShe\s)|(?=\sThat\s)|(?=\sThe\s)|(?=\sThere\s)|(?=\sThey\s)|(?=\sWe\s)|(?=\sWhat\s)|(?=\sWhen\s)|(?=\sWhere\s)|(?=\sWho\s)|(?=\sWhy\s))",
                r"\1.",
            )?,
        })
    }

    pub fn replace(&self, text: &str) -> SegmenterResult<String> {
        let text = self.possessive_abbreviation_rule.replace_all(&text);
        let mut text = self.kommanditgesellschaft_rule.replace_all(&text);
        for rule in &self.single_letter_abbreviation_rules {
            text = rule.replace_all(&text);
        }

        let text = {
            // NOTE: 이 부분 pySBD와 원본 루비 구현체 (pragmatic-segmenter)의
            // 동작이 전혀 다른데, pySBD를 따라간다.
            let mut abbr_handled_text = String::new();
            for line in self.python_splitlines_keepends.splitlines_keepends(&text) {
                abbr_handled_text += &self.search_for_abbreviations_in_string(line)?;
            }
            abbr_handled_text
        };

        // replace_multi_period_abbreviations()
        let mut text = self
            .multi_period_abbreviation_regex
            .replace_all(&text, |c: &Captures| {
                let mat = c.at(0).unwrap(); // Must exists
                self.multi_period_abbreviation_replace_period
                    .replace_all(mat)
            });

        for rule in &self.am_pm_rules {
            text = rule.replace_all(&text);
        }

        // replace_abbreviation_as_sentence_boundary()
        let text = self
            .replace_abbreviation_as_sentence_boundary
            .replace_all(&text);

        Ok(text)
    }

    fn search_for_abbreviations_in_string<'a>(
        &self,
        text: &'a str,
    ) -> SegmenterResult<Cow<'a, str>> {
        let lowered = text.to_lowercase();

        let mut text = Cow::Borrowed(text);
        for (abbr, abbr_match_regex, next_word_start_regex) in &self.abbreviations {
            if !lowered.contains(abbr) {
                continue;
            }
            let abbrev_match: Vec<_> = abbr_match_regex.find_iter(&text).collect();
            if abbrev_match.is_empty() {
                continue;
            }
            let char_array: Vec<_> = next_word_start_regex.find_iter(&text).collect();

            for (ind, range) in abbrev_match.into_iter().enumerate() {
                let abbr = &text[range.0..range.1].trim();

                // scan_for_replacements()
                let ch = char_array.get(ind).map(|r| &text[r.0..r.1]).unwrap_or("");

                // NOTE: 파이썬 구현체와 루비 구현체의 동작이 전혀 다르다. 루비 구현체에서는
                // uppercase letter가 단 한개라도 있으면 upper가 true가 되도록 구현되어있는데,
                // 파이썬 구현체에서는 모든 cased letter가 uppercase여야만 true가 되도록
                // 구현되어있다. 여기에선 pySBD와 동일하게 동작하도록 구현한다.
                //
                // References:
                //   https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/abbreviation_replacer.py#L104
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c/lib/pragmatic_segmenter/abbreviation_replacer.rb#L51
                let upper = python_isupper(ch);
                let abbr_lower = abbr.to_lowercase();
                let abbr_lower = abbr_lower.as_str();
                let is_prepositive = self.prepositive_abbreviations.contains(abbr_lower);
                if !upper || is_prepositive {
                    // NOTE: abbr에 escape를 해주는것이 맞으나, pySBD에 그런 처리가 되어있지 않다.
                    // pySBD와 동작을 맞추기 위해, 버그를 의도적으로 유지한다.
                    let regex = if is_prepositive {
                        // replace_prepositive_abbr()
                        format!(r"(?<=\s{abbr})\.(?=(\s|:\d+))", abbr = abbr)
                    } else if self.number_abbreviations.contains(abbr_lower) {
                        // replace_pre_number_abbr()
                        format!(r"(?<=\s{abbr})\.(?=(\s\d|\s+\())", abbr = abbr)
                    } else {
                        // replace_period_of_abbr()
                        format!(
                            r"(?<=\s{abbr})\.(?=((\.|\:|-|\?|,)|(\s([a-z]|I\s|I'm|I'll|\d|\())))",
                            abbr = abbr
                        )
                    };

                    // prepend a space to avoid needing another regex for start of string
                    let txt = format!(" {}", text);
                    // NOTE: Regex compile을 string match 도중에 하면 성능에 좋지 않지만, abbr이
                    // 동적이어서 어쩔 수 없다.
                    let txt = re(&regex)?.replace_all(&txt, "∯");
                    // remove the prepended space
                    text = Cow::Owned(txt[1..].to_string());
                }
            }
        }

        Ok(text)
    }
}

/// Rust implementation of Python's [`str.splitlines(keepends=True)`][ref].
///
/// [ref]: https://docs.python.org/3/library/stdtypes.html#str.splitlines
struct PythonSplitLines(AhoCorasick);

impl PythonSplitLines {
    fn new() -> Self {
        let newlines = &[
            "\r\n",     // Carriage Return + Line Feed
            "\n",       // Line Feed
            "\r",       // Carriage Return
            "\x0b",     // Line Tabulation
            "\x0c",     // Form Feed
            "\x1c",     // File Separator
            "\x1d",     // Group Separator
            "\x1e",     // Record Separator
            "\u{85}",   // Next Line (C1 Control Code)
            "\u{2028}", // Line Separator
            "\u{2029}", // Paragraph Separator
        ];

        Self(
            AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostFirst)
                .dfa(true)
                .build(newlines),
        )
    }

    fn splitlines_keepends<'a>(&self, input: &'a str) -> PythonSplitLinesKeepEnds<'_, 'a> {
        PythonSplitLinesKeepEnds {
            input,
            last_index: 0,
            searcher: self.0.find_iter(input),
        }
    }
}

struct PythonSplitLinesKeepEnds<'ac, 'input> {
    input: &'input str,
    last_index: usize,
    searcher: FindIter<'ac, 'input, usize>,
}

impl<'ac, 'input> Iterator for PythonSplitLinesKeepEnds<'ac, 'input> {
    type Item = &'input str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.searcher.next() {
            Some(mat) => {
                let begin = self.last_index;
                let end = mat.end();
                self.last_index = end;
                Some(&self.input[begin..end])
            }
            None => {
                let last = self.last_index;
                let len = self.input.len();
                if last < len {
                    self.last_index = len;
                    Some(&self.input[last..len])
                } else {
                    None
                }
            }
        }
    }
}

#[test]
fn test_python_splitlines_keepends() {
    let splitter = PythonSplitLines::new();

    let input = "x\nx\rx\r\nx\x0bx\x0cx\x1cx\x1dx\x1ex\u{85}x\u{2028}x\u{2029}";
    let output = [
        "x\n",
        "x\r",
        "x\r\n",
        "x\x0b",
        "x\x0c",
        "x\x1c",
        "x\x1d",
        "x\x1e",
        "x\u{85}",
        "x\u{2028}",
        "x\u{2029}",
    ];
    assert_eq!(
        splitter.splitlines_keepends(input).collect::<Vec<_>>(),
        output
    );

    let input = "\n\na";
    let output = ["\n", "\n", "a"];
    assert_eq!(
        splitter.splitlines_keepends(input).collect::<Vec<_>>(),
        output
    );
}

/// Rust implementation of Python's [`str.isupper()`][ref].
///
/// [ref]: https://docs.python.org/3/library/stdtypes.html#str.isupper
///
/// Reference: https://github.com/RustPython/RustPython/pull/1577
fn python_isupper(text: &str) -> bool {
    let mut cased = false;
    for c in text.chars() {
        if is_cased(c) && c.is_uppercase() {
            cased = true
        } else if is_cased(c) && c.is_lowercase() {
            return false;
        }
    }
    cased
}

#[test]
fn test_python_isupper() {
    assert!(!python_isupper("abc"));
    assert!(!python_isupper("123"));
    assert!(python_isupper("A_B"));
    assert!(!python_isupper("a_b"));
    assert!(python_isupper("A1"));
    assert!(python_isupper("1A"));
    assert!(!python_isupper("a1"));
    assert!(!python_isupper("1a"));
    assert!(!python_isupper("가나다a"));
    assert!(python_isupper("가나다A"));
}