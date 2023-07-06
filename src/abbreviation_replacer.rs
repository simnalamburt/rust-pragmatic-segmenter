use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::iter::Iterator;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, AhoCorasickKind, FindIter, MatchKind};
use onig::{Captures, Error, Regex};
use unic_ucd_case::is_cased;

use crate::rule::Rule;
use crate::util::{re, re_i};

pub struct AbbreviationReplacer {
    possessive_abbreviation_rule: Rule,
    kommanditgesellschaft_rule: Rule,
    single_letter_abbreviation_rules: [Rule; 2],
    am_pm_rules: [Rule; 4],

    python_splitlines_keepends: PythonSplitLines,

    abbreviations: Vec<(&'static str, Regex, Regex)>,
    prepositive_abbreviations: HashSet<&'static str>,
    number_abbreviations: HashSet<&'static str>,

    multi_period_abbreviation_regex: Regex,

    replace_abbreviation_as_sentence_boundary: Rule,
}

// NOTE: 이 글자들은 regex 안에 들어간다. ABBREVIATIONS를 고칠경우 특수문자를 사용하지 않도록
// 유의하고, 특수문자를 써야할경우 ABBREVIATIONS가 사용되는곳의 코드를 모두 함께 고쳐야한다.
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
    pub fn new() -> Result<Self, Error> {
        Ok(AbbreviationReplacer {
            // Example: https://rubular.com/r/yqa4Rit8EY
            possessive_abbreviation_rule: Rule::new(r"\.(?='s\s)|\.(?='s$)|\.(?='s\Z)", "∯")?,

            // Example: https://rubular.com/r/NEv265G2X2
            kommanditgesellschaft_rule: Rule::new(r"(?<=Co)\.(?=\sKG)", "∯")?,

            single_letter_abbreviation_rules: [
                // SingleUpperCaseLetterAtStartOfLineRule
                // Example: https://rubular.com/r/e3H6kwnr6H
                Rule::new(r"(?<=^[A-Z])\.(?=\s)", "∯")?,
                // SingleUpperCaseLetterRule
                // Example: https://rubular.com/r/gitvf0YWH4
                Rule::new(r"(?<=\s[A-Z])\.(?=,?\s)", "∯")?,
            ],

            am_pm_rules: [
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
                .map(|&abbr| -> Result<_, Error> {
                    // NOTE: 여기에서도 escaped이 된 abbr을 써야하지만, pySBD와 동작을 유지하기위해
                    // 의도적으로 abbr를 바로 사용한다
                    //
                    // NOTE: 이 Regex의 match 결과물이 다른 regex의 일부로 들어가게된다. 이 regex를
                    // 고칠경우 search_for_abbreviations_in_string() 함수에서 regex를 컴파일한 뒤
                    // unwrap()했던 부분이 영향받을 수 있다.
                    let abbr_match = re_i(&format!(r"(?:^|\s|\r|\n){}", abbr))?;

                    // NOTE: abbr에 . 이외의 글자가 들어가게될 경우, 아래의 escape 로직도 함께
                    // 고쳐야한다.
                    let escaped = abbr.replace('.', r"\.");
                    let next_word_start = re(&format!(r"(?<={{{}}} ).{{1}}", escaped))?;

                    Ok((abbr, abbr_match, next_word_start))
                })
                .collect::<Result<_, _>>()?,

            prepositive_abbreviations: PREPOSITIVE_ABBREVIATIONS.iter().copied().collect(),
            number_abbreviations: NUMBER_ABBREVIATIONS.iter().copied().collect(),

            // Example: https://rubular.com/r/xDkpFZ0EgH
            multi_period_abbreviation_regex: re_i(r"\b[a-z](?:\.[a-z])+[.]")?,

            replace_abbreviation_as_sentence_boundary: Rule::new(
                r"(U∯S|U\.S|U∯K|E∯U|E\.U|U∯S∯A|U\.S\.A|I|i.v|I.V)∯((?=\sA\s)|(?=\sBeing\s)|(?=\sDid\s)|(?=\sFor\s)|(?=\sHe\s)|(?=\sHow\s)|(?=\sHowever\s)|(?=\sI\s)|(?=\sIn\s)|(?=\sIt\s)|(?=\sMillions\s)|(?=\sMore\s)|(?=\sShe\s)|(?=\sThat\s)|(?=\sThe\s)|(?=\sThere\s)|(?=\sThey\s)|(?=\sWe\s)|(?=\sWhat\s)|(?=\sWhen\s)|(?=\sWhere\s)|(?=\sWho\s)|(?=\sWhy\s))",
                r"\1.",
            )?,
        })
    }

    pub fn replace(&self, text: &str) -> String {
        let text = self.possessive_abbreviation_rule.replace_all(text);
        let mut text = self.kommanditgesellschaft_rule.replace_all(&text);
        for rule in &self.single_letter_abbreviation_rules {
            text = rule.replace_all(&text);
        }

        let text = {
            // NOTE: 이 부분 pySBD와 원본 루비 구현체 (pragmatic-segmenter)의
            // 동작이 전혀 다른데, pySBD를 따라간다.
            let mut abbr_handled_text = String::new();
            for line in self.python_splitlines_keepends.splitlines_keepends(&text) {
                abbr_handled_text += &self.search_for_abbreviations_in_string(line);
            }
            abbr_handled_text
        };

        // replace_multi_period_abbreviations()
        let mut text = self
            .multi_period_abbreviation_regex
            .replace_all(&text, |c: &Captures| {
                let mat = c.at(0).unwrap(); // Must exists
                mat.replace('.', "∯")
            });

        for rule in &self.am_pm_rules {
            text = rule.replace_all(&text);
        }

        // replace_abbreviation_as_sentence_boundary()
        self.replace_abbreviation_as_sentence_boundary
            .replace_all(&text)
    }

    fn search_for_abbreviations_in_string<'a>(&self, text: &'a str) -> Cow<'a, str> {
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

            let mut replace_locations = BTreeSet::new();
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
                    let prepended_text = format!(" {}", text);

                    // NOTE: Regex compile을 string match 도중에 하면 성능에 좋지 않지만, abbr이
                    // 동적이어서 어쩔 수 없이 여기서 compile을 수행한다.
                    //
                    // NOTE: 현재 구현상 abbr은 무조건 ABBREVIATIONS의 일부이기때문에 여기서
                    // unwrap()해도 안전하다. 그러나 구현이 바뀔경우 조치가 필요하다.
                    replace_locations.extend(re(&regex).unwrap().find_iter(&prepended_text).map(
                        // 맨 앞에 스페이스바를 붙였기때문에 1 뺴야함
                        |r| r.0 - 1,
                    ));
                    // TODO: replace_locations에 같은 인덱스를 중복으로 추가하기때문에, 비효율이
                    // 발생함. 최적화하기.
                }
            }

            if !replace_locations.is_empty() {
                let mut owned = text.into_owned();
                for loc in replace_locations.into_iter().rev() {
                    owned.replace_range(loc..(loc + 1), "∯");
                }
                text = Cow::Owned(owned);
            }
        }

        text
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
                .kind(Some(AhoCorasickKind::DFA))
                .build(newlines)
                .unwrap(), // NOTE: It does not fails with our small input
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
    searcher: FindIter<'ac, 'input>,
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

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Error>;

    #[test]
    fn regex_should_be_compiled() {
        assert!(AbbreviationReplacer::new().is_ok())
    }

    #[test]
    fn test_abbr_replace() -> TestResult {
        let rep = AbbreviationReplacer::new()?;

        assert_eq!(
            rep.replace("Humana Inc. is including"),
            "Humana Inc∯ is including"
        );

        Ok(())
    }

    #[test]
    fn test_search_for_abbreviations_in_string() -> TestResult {
        let rep = AbbreviationReplacer::new()?;

        assert_eq!(
            rep.search_for_abbreviations_in_string("Humana Inc. is including"),
            "Humana Inc∯ is including"
        );

        Ok(())
    }
}
