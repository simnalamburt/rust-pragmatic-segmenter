mod abbreviation_replacer;
mod list_item_replacer;
mod rule;
mod util;

use std::error::Error;
use std::iter::Iterator;

use onig::{Captures, Regex};

use abbreviation_replacer::AbbreviationReplacer;
use list_item_replacer::ListItemReplacer;
use rule::Rule;
use util::re;

type SegmenterResult<T> = Result<T, Box<dyn Error>>;

const PUNCTUATIONS: [char; 7] = ['。', '．', '.', '！', '!', '?', '？'];

pub struct Segmenter {
    list_item_replacer: ListItemReplacer,
    abbreviation_replacer: AbbreviationReplacer,

    number_rules: [Rule; 5],
    continuous_punctuation_regex: Regex,
    numbered_reference: Rule,
    misc_rules: [Rule; 3],

    parens_between_double_quotes_regex: Regex,
    parens_between_double_quotes_0: Rule,
    parens_between_double_quotes_1: Rule,

    ellipsis_rules: [Rule; 5],

    exclamation_regex: Regex,
    sub_escaped_regex_reserved_characters: [Rule; 5],

    word_with_leading_apostrophe: Regex,
    trailing_apostrophe: Regex,
    between_single_quotes_regex: Regex,
    between_single_quote_slanted_regex: Regex,
    between_double_quotes_regex_2: Regex,
    between_square_brackets_regex_2: Regex,
    between_parens_regex_2: Regex,
    between_quote_arrow_regex_2: Regex,
    between_em_dashes_regex_2: Regex,
    between_quote_slanted_regex_2: Regex,

    double_punctuation: Regex,
    question_mark_in_quotation_and_exclamation_point_rules: [Rule; 4],

    replace_parens: Rule,

    sentence_boundary_regex: Regex,
    post_process_regex: Regex,
    quotation_at_end_of_sentence_regex: Regex,
}

impl Segmenter {
    pub fn new() -> SegmenterResult<Self> {
        Ok(Segmenter {
            list_item_replacer: ListItemReplacer::new()?,
            abbreviation_replacer: AbbreviationReplacer::new()?,

            number_rules: [
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

            misc_rules: [
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

            // Example: https://rubular.com/r/6flGnUMEVl
            parens_between_double_quotes_regex: re(r#"["\”]\s\(.*\)\s["\“]"#)?,
            parens_between_double_quotes_0: Rule::new(r"\s(?=\()", "\r")?,
            parens_between_double_quotes_1: Rule::new(r"(?<=\))\s", "\r")?,

            // NOTE: 이부분은 pySBD 구현과 루비 구현이 동작이 다르다. pySBD의 동작을 따른다.
            // 이 부분을 고치게 되면 ReinsertEllipsisRules도 함께 고쳐야한다.
            ellipsis_rules: [
                // ThreeSpaceRule
                // Example: https://rubular.com/r/YBG1dIHTRu
                Rule::new(r"(\s\.){3}\s", "♟♟♟♟♟♟♟")?,
                // FourSpaceRule
                // Example: https://rubular.com/r/2VvZ8wRbd8
                Rule::new(r"(?<=[a-z])(\.\s){3}\.($|\\n)", "♝♝♝♝♝♝♝")?,
                // FourConsecutiveRule
                // Example: https://rubular.com/r/Hdqpd90owl
                Rule::new(r"(?<=\S)\.{3}(?=\.\s[A-Z])", "ƪƪƪ")?,
                // ThreeConsecutiveRule
                // Example: https://rubular.com/r/i60hCK81fz
                Rule::new(r"\.\.\.(?=\s+[A-Z])", "☏☏.")?,
                // OtherThreePeriodRule
                Rule::new(r"\.\.\.", "ƪƪƪ")?,
            ],

            exclamation_regex: re(
                r"!Xũ|!Kung|ǃʼOǃKung|!Xuun|!Kung\-Ekoka|ǃHu|ǃKhung|ǃKu|ǃung|ǃXo|ǃXû|ǃXung|ǃXũ|!Xun|Yahoo!|Y!J|Yum!",
            )?,

            // NOTE: pySBD에 구현 실수가 있어 루비 구현체와 동작이 전혀 다르지만, pySBD의 동작을
            // 따르기 위해 버그를 유지하겠다.
            sub_escaped_regex_reserved_characters: [
                // SubLeftParen
                Rule::new(r"\\\(", "(")?,
                // SubRightParen
                Rule::new(r"\\\)", ")")?,
                // SubLeftBracket
                Rule::new(r"\\\[", "[")?,
                // SubRightBracket
                Rule::new(r"\\\]", "]")?,
                // SubDash
                Rule::new(r"\\\-", "-")?,
            ],

            // Example: https://rubular.com/r/mXf8cW025o
            word_with_leading_apostrophe: re(r"(?<=\s)'(?:[^']|'[a-zA-Z])*'\S")?,

            trailing_apostrophe: re(r"'\s")?,

            // Example: https://rubular.com/r/2YFrKWQUYi
            between_single_quotes_regex: re(r"(?<=\s)'(?:[^']|'[a-zA-Z])*'")?,

            between_single_quote_slanted_regex: re(r"(?<=\s)‘(?:[^’]|’[a-zA-Z])*’")?,

            // Example: https://regex101.com/r/r6I1bW/1
            //
            // NOTE: pySBD에선 파이썬 regex의 기능 한계로 인해 원본인 루비 pragmatic_segmenter와
            // 동작이 다른데, 우리는 Oniguruma regex engine을 쓰고있으므로 루비 구현을 재현할 수
            // 있다. 그러나 pySBD와 동작을 맞추기 위해 의도적으로 pySBD 정규표현식을 사용한다.
            //
            // NOTE: Python regex와 Oniguruma regex는 named capture group과 backreference 문법이
            // 다르다. 주의
            //
            // Reference: https://stackoverflow.com/a/13577411/13977061
            between_double_quotes_regex_2: re(r#""(?=(?<tmp>[^\"\\]+|\\{2}|\\.)*)\k<tmp>""#)?,
            between_square_brackets_regex_2: re(r#"\[(?=(?<tmp>[^\]\\]+|\\{2}|\\.)*)\k<tmp>\]"#)?,
            between_parens_regex_2: re(r"\((?=(?<tmp>[^\(\)\\]+|\\{2}|\\.)*)\k<tmp>\)")?,
            between_quote_arrow_regex_2: re(r"\«(?=(?<tmp>[^»\\]+|\\{2}|\\.)*)\k<tmp>\»")?,
            between_em_dashes_regex_2: re(r"--(?=(?<tmp>[^--]*))\k<tmp>--")?,
            between_quote_slanted_regex_2: re(r"\“(?=(?<tmp>[^”\\]+|\\{2}|\\.)*)\k<tmp>\”")?,

            double_punctuation: re(r"^(?:\?!|!\?|\?\?|!!)")?,
            question_mark_in_quotation_and_exclamation_point_rules: [
                // QuestionMarkInQuotationRule
                // Example: https://rubular.com/r/aXPUGm6fQh
                Rule::new(r#"\?(?=(\'|\"))"#, "&ᓷ&")?,
                // InQuotationRule
                // Example: https://rubular.com/r/XS1XXFRfM2
                Rule::new(r#"\!(?=(\'|\"))"#, "&ᓴ&")?,
                // BeforeCommaMidSentenceRule
                // Example: https://rubular.com/r/sl57YI8LkA
                Rule::new(r"\!(?=\,\s[a-z])", "&ᓴ&")?,
                // MidSentenceRule
                // Example: https://rubular.com/r/f9zTjmkIPb
                Rule::new(r"\!(?=\s[a-z])", "&ᓴ&")?,
            ],

            // Example: https://rubular.com/r/GcnmQt4a3I
            replace_parens: Rule::new(
                // ROMAN_NUMERALS_IN_PARENTHESES
                r"\(((?=[mdclxvi])m*(c[md]|d?c*)(x[cl]|l?x*)(i[xv]|v?i*))\)(?=\s[A-Z])",
                r"&✂&\1&⌬&",
            )?,

            // added special case: r"[。．.！!?].*" to handle intermittent dots, exclamation, etc.
            sentence_boundary_regex: re(
                r#"（(?:[^）])*）(?=\s?[A-Z])|「(?:[^」])*」(?=\s[A-Z])|\((?:[^\)]){2,}\)(?=\s[A-Z])|\'(?:[^\'])*[^,]\'(?=\s[A-Z])|\"(?:[^\"])*[^,]\"(?=\s[A-Z])|\“(?:[^\”])*[^,]\”(?=\s[A-Z])|[。．.！!?？].*|\S.*?[。．.！!?？ȸȹ☉☈☇☄]"#,
            )?,
            post_process_regex: re(r"\A[a-zA-Z]*\Z")?,
            // Example: https://rubular.com/r/NqCqv372Ix
            quotation_at_end_of_sentence_regex: re(r#"[!?\.-][\"\'“”]\s{1}[A-Z]"#)?,
        })
    }

    pub fn segment<'a>(&'a self, text: &str) -> impl Iterator<Item = String> + 'a {
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

        //
        // split_into_segments()
        //

        // check_for_parens_between_quotes()
        let text = self
            .parens_between_double_quotes_regex
            .replace_all(&text, |c: &Captures| {
                let mat = c.at(0).unwrap(); // Must exists
                let mat = self.parens_between_double_quotes_0.replace_all(mat);
                let mat = self.parens_between_double_quotes_1.replace_all(&mat);
                mat
            });

        // TODO: flat_map() 에서 임시 Vec, String 할당 줄이기
        text.split('\r')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>() // String을 own하는 버전의 새 split 함수를 만들면 이부분을 제거할 수 있음
            .into_iter()
            .flat_map(move |sent| {
                // English.SingleNewLineRule
                let mut sent = sent.replace(r"\n", "ȹ");
                // English.EllipsisRules.All
                for rule in &self.ellipsis_rules {
                    sent = rule.replace_all(&sent);
                }
                // check_for_punctuation()
                if PUNCTUATIONS.iter().any(|&p| sent.contains(p)) {
                    // process_text()
                    if !sent.ends_with(&PUNCTUATIONS[..]) {
                        sent += "ȸ";
                    }

                    // ExclamationWords.apply_rules()
                    sent = self
                        .exclamation_regex
                        .replace_all(&sent, self.replace_punctuation(false));

                    // between_punctuation()
                    if self.word_with_leading_apostrophe.find(&sent).is_none()
                        || self.trailing_apostrophe.find(&sent).is_some()
                    {
                        sent = self
                            .between_single_quotes_regex
                            .replace_all(&sent, self.replace_punctuation(true));
                    }
                    sent = self
                        .between_single_quote_slanted_regex
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_double_quotes_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_square_brackets_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_parens_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_quote_arrow_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_em_dashes_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));
                    sent = self
                        .between_quote_slanted_regex_2
                        .replace_all(&sent, self.replace_punctuation(false));

                    // handle text having only doublepunctuations
                    if self.double_punctuation.find(&sent).is_none() {
                        sent = sent
                            .replace(r"?!", "☉")
                            .replace(r"!?", "☈")
                            .replace(r"??", "☇")
                            .replace(r"!!", "☄");
                    }
                    for rule in &self.question_mark_in_quotation_and_exclamation_point_rules {
                        sent = rule.replace_all(&sent);
                    }

                    // ListItemReplacer(sent).replace_parens()
                    sent = self.replace_parens.replace_all(&sent);

                    // sentence_boundary_punctuation()
                    // retain exclamation mark if it is an ending character of a given text
                    sent = sent.replace(r"&ᓴ&$", "!");
                    self.sentence_boundary_regex
                        .find_iter(&sent)
                        .map(|r| sent[r.0..r.1].to_string())
                        .collect::<Vec<_>>()
                } else {
                    vec![sent]
                }
            })
            .flat_map(move |mut sent| {
                // SubSymbolsRules
                sent = sent
                    .replace(r"∯", ".")
                    .replace(r"♬", "،")
                    .replace(r"♭", ":")
                    .replace(r"&ᓰ&", "。")
                    .replace(r"&ᓱ&", "．")
                    .replace(r"&ᓳ&", "！")
                    .replace(r"&ᓴ&", "!")
                    .replace(r"&ᓷ&", "?")
                    .replace(r"&ᓸ&", "？")
                    .replace(r"☉", "?!")
                    .replace(r"☇", "??")
                    .replace(r"☈", "!?")
                    .replace(r"☄", "!!")
                    .replace(r"&✂&", "(")
                    .replace(r"&⌬&", ")")
                    .replace(r"ȸ", "")
                    .replace(r"ȹ", "\n");

                // post_process_segments()
                //
                // NOTE: post_process_segments 함수는 pySBD와 루비 pragmatic_segmenter의 동작이 전혀
                // 다르다. pySBD를 따라간다.
                if sent.len() > 2 && self.post_process_regex.find(&sent).is_some() {
                    return vec![sent];
                }

                // ReinsertEllipsisRules
                // NOTE: 이부분은 pySBD 구현과 루비 구현이 동작이 다르다. pySBD의 동작을 따른다.
                sent = sent
                    .replace(r"ƪƪƪ", "...")
                    .replace(r"♟♟♟♟♟♟♟", " . . . ")
                    .replace(r"♝♝♝♝♝♝♝", ". . . .")
                    .replace(r"☏☏", "..")
                    .replace(r"∮", ".");

                if self
                    .quotation_at_end_of_sentence_regex
                    .find(&sent)
                    .is_some()
                {
                    self.quotation_at_end_of_sentence_regex
                        .split(&sent)
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![sent.replace("\n", "").trim().to_string()]
                }
            })
            .map(|sent| sent.replace(r"&⎋&", "'"))
    }

    fn replace_punctuation(&self, is_match_type_single: bool) -> impl Fn(&Captures) -> String + '_ {
        move |c: &Captures| {
            let mat = c.at(0).unwrap(); // Must exists

            // NOTE: 원래 이 자리에서 EscapeRegexReservedCharacters.All 규칙이 적용되어야
            // 하나, pySBD의 구현 버그로 인해 EscapeRegexReservedCharacters.All가 아무일도
            // 하지 않는다. 버그이지만, pySBD의 동작을 따라가기위해 버그를 유지하겠다.

            let mut mat = mat.replace('.', "∯");
            mat = mat.replace('。', "&ᓰ&");
            mat = mat.replace('．', "&ᓱ&");
            mat = mat.replace('！', "&ᓳ&");
            mat = mat.replace('!', "&ᓴ&");
            mat = mat.replace('?', "&ᓷ&");
            mat = mat.replace('？', "&ᓸ&");
            if !is_match_type_single {
                mat = mat.replace("'", "&⎋&");
            }
            for rule in &self.sub_escaped_regex_reserved_characters {
                mat = rule.replace_all(&mat);
            }
            mat
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn regex_should_be_compiled() -> TestResult {
        let _seg = Segmenter::new()?;
        Ok(())
    }

    #[test]
    fn empty_string() -> TestResult {
        let seg = Segmenter::new()?;

        let expected: [String; 0] = [];
        let actual: Vec<_> = seg.segment("").collect();
        assert_eq!(actual, expected);
        Ok(())
    }
}
