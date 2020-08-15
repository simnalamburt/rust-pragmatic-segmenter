use std::borrow::Cow;
use std::collections::HashMap;

use onig::{Captures, Regex};

use crate::rule::Rule;
use crate::util::{re, re_i};
use crate::SegmenterResult;

pub struct ListItemReplacer {
    roman_numerals: HashMap<&'static str, isize>,
    latin_numerals: HashMap<&'static str, isize>,

    alphabetical_list_with_periods: Regex,
    alphabetical_list_with_parens: Regex,

    alphabetical_list_letters_and_periods_regex: Regex,
    extract_alphabetical_list_letters_regex: Regex,

    numbered_list_regex_1: Regex,
    numbered_list_regex_2: Regex,
    numbered_list_parens_regex: Regex,

    find_numbered_list_1: Regex,
    find_numbered_list_2: Regex,

    space_between_list_items_first_rule: Rule,
    space_between_list_items_second_rule: Rule,

    find_numbered_list_parens: Regex,

    space_between_list_items_third_rule: Rule,

    substitute_list_period_rule: Rule,
    list_marker_rule: Rule,
}

impl ListItemReplacer {
    #[must_use]
    pub fn new() -> SegmenterResult<Self> {
        #[must_use]
        fn map_from_list(list: &[&'static str]) -> HashMap<&'static str, isize> {
            list.iter()
                .enumerate()
                .map(|(idx, &s)| (s, idx as isize))
                .collect()
        }

        Ok(ListItemReplacer {
            roman_numerals: map_from_list(&[
                "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii",
                "xiv", "x", "xi", "xii", "xiii", "xv", "xvi", "xvii", "xviii", "xix", "xx",
            ]),

            latin_numerals: map_from_list(&[
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ]),

            // Example: https://rubular.com/r/XcpaJKH0sz
            //
            // NOTE: 루비 버전은 case sensitive하고, 파이썬 버전은 case insensitive한데, 루비
            // 버전에서 case sensitive하게 만들어진것이 실수같음. Case insensitive하게 만든다.
            alphabetical_list_with_periods: re_i(
                r"(?<=^)[a-z](?=\.)|(?<=\A)[a-z](?=\.)|(?<=\s)[a-z](?=\.)",
            )?,

            // Example: https://rubular.com/r/Gu5rQapywf
            alphabetical_list_with_parens: re_i(
                r"(?<=\()[a-z]+(?=\))|(?<=^)[a-z]+(?=\))|(?<=\A)[a-z]+(?=\))|(?<=\s)[a-z]+(?=\))",
            )?,

            // Example: https://rubular.com/r/wMpnVedEIb
            alphabetical_list_letters_and_periods_regex: re_i(
                r"(?<=^)[a-z]\.|(?<=\A)[a-z]\.|(?<=\s)[a-z]\.",
            )?,

            // Example: https://rubular.com/r/NsNFSqrNvJ
            extract_alphabetical_list_letters_regex: re_i(
                r"\([a-z]+(?=\))|(?<=^)[a-z]+(?=\))|(?<=\A)[a-z]+(?=\))|(?<=\s)[a-z]+(?=\))",
            )?,

            // Example: https://regex101.com/r/cd3yNz/2
            numbered_list_regex_1: re(
                r"\s\d{1,2}(?=\.\s)|^\d{1,2}(?=\.\s)|\s\d{1,2}(?=\.\))|^\d{1,2}(?=\.\))|(?<=\s\-)\d{1,2}(?=\.\s)|(?<=^\-)\d{1,2}(?=\.\s)|(?<=\s\⁃)\d{1,2}(?=\.\s)|(?<=^\⁃)\d{1,2}(?=\.\s)|(?<=s\-)\d{1,2}(?=\.\))|(?<=^\-)\d{1,2}(?=\.\))|(?<=\s\⁃)\d{1,2}(?=\.\))|(?<=^\⁃)\d{1,2}(?=\.\))",
            )?,

            // Example: https://regex101.com/r/cd3yNz/1
            numbered_list_regex_2: re(
                r"(?<=\s)\d{1,2}\.(?=\s)|^\d{1,2}\.(?=\s)|(?<=\s)\d{1,2}\.(?=\))|^\d{1,2}\.(?=\))|(?<=\s\-)\d{1,2}\.(?=\s)|(?<=^\-)\d{1,2}\.(?=\s)|(?<=\s\⁃)\d{1,2}\.(?=\s)|(?<=^\⁃)\d{1,2}\.(?=\s)|(?<=\s\-)\d{1,2}\.(?=\))|(?<=^\-)\d{1,2}\.(?=\))|(?<=\s\⁃)\d{1,2}\.(?=\))|(?<=^\⁃)\d{1,2}\.(?=\))",
            )?,

            // Example: https://regex101.com/r/O8bLbW/1
            numbered_list_parens_regex: re(r"\d{1,2}(?=\)\s)")?,

            // TODO: lookaround 없음
            //
            // Reference: https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/lists_item_replacer.py#L143
            find_numbered_list_1: re(r"♨.+\n.+♨|♨.+\r.+♨")?,

            // TODO: lookaround 없음
            //
            // Reference: https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/lists_item_replacer.py#L144
            find_numbered_list_2: re(r"for\s\d{1,2}♨\s[a-z]")?,

            // NOTE: pySBD와 pragmatic-segmenter(루비 구현체)가 다른 regex를 씀, pySBD를 따라감
            //
            // Example:
            //   https://rubular.com/r/Wv4qLdoPx7
            //   https://regex101.com/r/62YBlv/1
            space_between_list_items_first_rule: Rule::new(r"(?<=\S\S)\s(?=\S\s*\d+♨)", "\r")?,

            // NOTE: pySBD와 pragmatic-segmenter(루비 구현체)가 다른 regex를 씀, pySBD를 따라감
            //
            // Example:
            //   https://rubular.com/r/AizHXC6HxK
            //   https://regex101.com/r/62YBlv/2
            space_between_list_items_second_rule: Rule::new(r"(?<=\S\S)\s(?=\d{1,2}♨)", "\r")?,

            // TODO: lookaround 없음
            //
            // Refernce: https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/lists_item_replacer.py#L154
            find_numbered_list_parens: re(r"☝.+\n.+☝|☝.+\r.+☝")?,

            // NOTE: pySBD와 pragmatic-segmenter(루비 구현체)가 다른 regex를 씀, pySBD를 따라감
            //
            // Example:
            //   https://rubular.com/r/GE5q6yID2j
            //   https://regex101.com/r/62YBlv/3
            space_between_list_items_third_rule: Rule::new(r"(?<=\S\S)\s(?=\d{1,2}☝)", "\r")?,

            // TODO: lookaround 없음
            substitute_list_period_rule: Rule::new("♨", "∯")?,
            // TODO: lookaround 없음
            list_marker_rule: Rule::new("☝", "")?,
        })
    }

    #[must_use]
    pub fn add_line_break<'a>(&self, text: &'a str) -> SegmenterResult<String> {
        // format_alphabetical_lists()
        let text =
            self.iterate_alphabet_array(&text, &self.alphabetical_list_with_periods, false, false);
        let text =
            self.iterate_alphabet_array(&text, &self.alphabetical_list_with_parens, true, false);

        // format_roman_numeral_lists()
        let text =
            self.iterate_alphabet_array(&text, &self.alphabetical_list_with_periods, false, true);
        let text =
            self.iterate_alphabet_array(&text, &self.alphabetical_list_with_parens, true, true);

        // format_numbered_list_with_periods()
        let text = self.scan_lists(
            &text,
            &self.numbered_list_regex_1,
            &self.numbered_list_regex_2,
            '♨',
            true,
        )?;
        let text = self.add_line_breaks_for_numbered_list_with_periods(&text);
        let text = self.substitute_list_period_rule.replace_all(&text);

        // format_numbered_list_with_parens()
        let text = self.scan_lists(
            &text,
            &self.numbered_list_parens_regex,
            &self.numbered_list_parens_regex,
            '☝',
            false,
        )?;
        let text = self.add_line_breaks_for_numbered_list_with_parens(&text);
        let text = self.list_marker_rule.replace_all(&text);

        Ok(text)
    }

    #[must_use]
    fn replace_alphabet_list(&self, text: &str, what_to_replace: &str) -> String {
        self.alphabetical_list_letters_and_periods_regex
            .replace_all(text, |m: &Captures| {
                let mat = m.at(0).unwrap(); // Must exists
                let match_wo_period = mat.strip_suffix('.').unwrap_or(mat);
                if match_wo_period == what_to_replace {
                    format!("\r{}∯", match_wo_period)
                } else {
                    mat.to_string()
                }
            })
    }

    #[must_use]
    fn replace_alphabet_list_parens(&self, text: &str, what_to_replace: &str) -> String {
        self.extract_alphabetical_list_letters_regex
            .replace_all(text, |m: &Captures| {
                let mat = m.at(0).unwrap(); // Must exists

                // NOTE: 루비코드에선 검사하기 전에 mat을 downcase 한다. 파이썬에선 안함. downcase
                // 하는것이 맞지만, 일단은 pySBD와 같은 동작을 만들겠다.
                //
                // Reference:
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c/lib/pragmatic_segmenter/list.rb#L149
                if let Some(match_wo_paren) = mat.strip_prefix('(') {
                    if match_wo_paren == what_to_replace {
                        format!("\r&✂&{}", match_wo_paren)
                    } else {
                        mat.to_string()
                    }
                } else {
                    if mat == what_to_replace {
                        format!("\r{}", mat)
                    } else {
                        mat.to_string()
                    }
                }
            })
    }

    #[must_use]
    fn iterate_alphabet_array<'a>(
        &self,
        text: &'a str,
        regex: &Regex,
        parens: bool,
        use_roman_numeral: bool,
    ) -> Cow<'a, str> {
        // NOTE: 루비 코드(pragmatic segmenter)에선 여기서 검사하기 전에 downcase를 함, pySBD에선
        // 안함. Downcase를 하는것이 맞지만, 이 프로젝트는 일단 pySBD의 동작을 따르겠다.
        //
        // Reference:
        //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491/lib/pragmatic_segmenter/list.rb#L186
        let alphabet = if use_roman_numeral {
            &self.roman_numerals
        } else {
            &self.latin_numerals
        };

        let list_array: Vec<_> = regex
            .find_iter(text)
            .filter_map(|x| alphabet.get(&text[x.0..x.1]).map(|&v| (&text[x.0..x.1], v)))
            .collect();

        let len = list_array.len();

        let mut result = Cow::Borrowed(text);
        for ind in 0..len {
            let is_strange = if len <= 1 {
                // NOTE: 원본 코드에선 len이 1이면 무조건 스킵하게 만들어져있고, 버그로 생각된다.
                // 그러나 그 동작을 유지하겠다.
                true
            } else if ind == len - 1 {
                (list_array[len - 2].1 - list_array[len - 1].1).abs() != 1
            } else if ind == 0 {
                // NOTE: 원본 코드에선 ind가 0인 경우를 고려하지 않는다. 이때문에 말도 안되는
                // 버그가 생기지만, 원본 코드의 동작을 유지하는것이 목표여서 버그를 유지하겠다.
                //
                // NOTE: 그리고 뺄셈 부분에서 일부만 abs를 쓰고 일부는 abs를 안쓰는데, 이것도
                // pySBD와 루비의 코드를 유지한 것이다.
                //
                // References:
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c/lib/pragmatic_segmenter/list.rb#L194
                //   https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/lists_item_replacer.py#L235
                list_array[1].1 - list_array[0].1 != 1
                    && (list_array[len - 1].1 - list_array[0].1).abs() != 1
            } else {
                list_array[ind + 1].1 - list_array[ind].1 != 1
                    && (list_array[ind - 1].1 - list_array[ind].1).abs() != 1
            };
            if is_strange {
                continue;
            }

            let each = list_array[ind].0;
            result = Cow::Owned(if parens {
                self.replace_alphabet_list_parens(&result, each)
            } else {
                self.replace_alphabet_list(&result, each)
            })
        }

        result
    }

    #[must_use]
    fn scan_lists<'a>(
        &self,
        text: &'a str,
        regex1: &Regex,
        regex2: &Regex,
        replacement: char,
        strip: bool,
    ) -> SegmenterResult<Cow<'a, str>> {
        let list_array: Vec<_> = regex1
            .find_iter(text)
            .map(|r| text[r.0..r.1].trim_start().parse::<i32>())
            .collect::<Result<_, _>>()?;

        let mut result = Cow::Borrowed(text);
        for (i, &each) in list_array.iter().enumerate() {
            if !(Some(&(each + 1)) == list_array.get(i + 1)
                || Some(&(each - 1)) == list_array.get(i - 1)
                || (each == 0 && list_array.get(i - 1) == Some(&9))
                || (each == 9 && list_array.get(i + 1) == Some(&0)))
            {
                continue;
            }

            // substitute_found_list_items()
            result = Cow::Owned(regex2.replace_all(&result, |m: &Captures| {
                let mut mat = m.at(0).unwrap(); // Must exists

                // NOTE: 원본 루비 코드 pragmatic-segmenter와 파이썬 구현체 pySBD의 동작이 다름.
                // pySBD의 동작을 따르겠다.
                //
                // Reference:
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c/lib/pragmatic_segmenter/list.rb#L112
                //   https://github.com/nipunsadvilkar/pySBD/blob/90699972/pysbd/lists_item_replacer.py#L112
                if strip {
                    mat = mat.trim();
                }
                let chomped = if mat.len() == 1 {
                    mat
                } else {
                    mat.trim_matches(&['.', ']', ')'][..])
                };
                if each.to_string() == chomped {
                    format!("{}{}", each, replacement)
                } else {
                    mat.to_string()
                }
            }))
        }

        Ok(result)
    }

    #[must_use]
    fn add_line_breaks_for_numbered_list_with_periods<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.contains('♨')
            && self.find_numbered_list_1.find(text).is_none()
            && self.find_numbered_list_2.find(text).is_none()
        {
            let text = self.space_between_list_items_first_rule.replace_all(text);
            let text = self.space_between_list_items_second_rule.replace_all(&text);
            return Cow::Owned(text);
        }

        Cow::Borrowed(text)
    }

    #[must_use]
    fn add_line_breaks_for_numbered_list_with_parens<'a>(&self, text: &'a str) -> Cow<'a, str> {
        if text.contains('☝') && self.find_numbered_list_parens.find(text).is_none() {
            let text = self.space_between_list_items_third_rule.replace_all(text);
            return Cow::Owned(text);
        }

        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    type TestResult = Result<(), Box<dyn Error>>;

    #[test]
    fn test_alphabetical_list_with_periods() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text =
            "a. The first item b. The second item c. The third list item D. case insesitive \
E. Don't select the nextF.dont't select this G should be followed by dot";

        assert_eq!(
            list.alphabetical_list_with_periods
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![
                (0, 1),   // a
                (18, 19), // b
                (37, 38), // c
                (60, 61), // D
                (79, 80), // E
            ]
        );
        Ok(())
    }

    #[test]
    fn test_alphabetical_list_with_parens() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text = "\
a) Hello world.
b) Hello world.
c) Hello world.
d) Hello world.
e) Hello world.
f) Hello world.

(i) Hello world.
(ii) Hello world.
(iii) Hello world.
(iv) Hello world.
(v) Hello world.
(vi) Hello world.
";

        assert_eq!(
            list.alphabetical_list_with_parens
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![
                (0, 1,),
                (16, 17,),
                (32, 33,),
                (48, 49,),
                (64, 65,),
                (80, 81,),
                (98, 99,),
                (115, 117,),
                (133, 136,),
                (152, 154,),
                (170, 171,),
                (187, 189,),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_alphabetical_list_letters_and_periods_regex() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text = "His name is Mark E. Smith. a. here it is b. another c. one more
 They went to the store. It was John A. Smith. She was Jane B. Smith.";

        assert_eq!(
            list.alphabetical_list_letters_and_periods_regex
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![
                (17, 19),   // "E."
                (27, 29),   // "a."
                (41, 43),   // "b."
                (52, 54),   // "c."
                (101, 103), // "A."
                (124, 126), // "B."
            ]
        );
        Ok(())
    }

    #[test]
    fn test_extract_alphabetical_list_letters_regex() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text =
        "a) here it is b) another c) one more \nThey went to the store. W) hello X) hello Y) hello";

        assert_eq!(
            list.extract_alphabetical_list_letters_regex
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![
                (0, 1),   // "a"
                (14, 15), // "b"
                (25, 26), // "c"
                (62, 63), // "W"
                (71, 72), // "X"
                (80, 81), // "Y"
            ]
        );
        Ok(())
    }

    #[test]
    fn test_numbered_list_regex_1() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text = "\
Match below

1.  abcd
2.  xyz
    1. as
    2. yo
3.  asdf
4.  asdf

Dont match below

1.abc
2) asdf
333. asdf
";

        assert_eq!(
            list.numbered_list_regex_1
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![(12, 14), (21, 23), (33, 35), (43, 45), (49, 51), (58, 60),]
        );

        Ok(())
    }

    #[test]
    fn test_numbered_list_regex_2() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text = "\
Match below

1.  abcd
2.  xyz
    1. as
    2. yo
3.  asdf
4.  asdf

Dont match below

1.abc
2) asdf
333. asdf
";

        assert_eq!(
            list.numbered_list_regex_2
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![(13, 15), (22, 24), (34, 36), (44, 46), (50, 52), (59, 61),]
        );

        Ok(())
    }

    #[test]
    fn test_numbered_list_parens_regex() -> TestResult {
        let list = ListItemReplacer::new()?;
        let text = "\
1) a
2) b
    1) b1
    2) b2
3) c
4) 5)
55) d
666) e
f77) f
8888) f
10)nomatch
-10) ignore sign
";

        assert_eq!(
            list.numbered_list_parens_regex
                .find_iter(text)
                .collect::<Vec<_>>(),
            vec![
                (0, 1),
                (5, 6),
                (14, 15),
                (24, 25),
                (30, 31),
                (35, 36),
                (38, 39),
                (41, 43),
                (48, 50),
                (55, 57),
                (63, 65),
                (81, 83),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_space_between_list_items_first_rule() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "abcd  ⁃9♨ The first item ⁃10♨ The second item ⁃9♨ The first item ⁃10♨ The second item ⁃9♨ The first item ⁃10♨ The second item ⁃9♨ The first item ⁃10♨ The second item ⁃9♨ The first item ⁃10♨ The second item ⁃9♨ The first item ⁃10♨ The second item";
        let output = "abcd  ⁃9♨ The first item\r⁃10♨ The second item\r⁃9♨ The first item\r⁃10♨ The second item\r⁃9♨ The first item\r⁃10♨ The second item\r⁃9♨ The first item\r⁃10♨ The second item\r⁃9♨ The first item\r⁃10♨ The second item\r⁃9♨ The first item\r⁃10♨ The second item";

        assert_eq!(
            list.space_between_list_items_first_rule.replace_all(input),
            output
        );

        Ok(())
    }

    #[test]
    fn test_space_between_list_items_second_rule() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "1♨ The first item 2♨ The second item";
        let output = "1♨ The first item\r2♨ The second item";

        assert_eq!(
            list.space_between_list_items_second_rule.replace_all(input),
            output
        );

        Ok(())
    }

    #[test]
    fn test_space_between_list_items_third_rule() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "1☝) The first item 2☝) The second item";
        let output = "1☝) The first item\r2☝) The second item";

        assert_eq!(
            list.space_between_list_items_third_rule.replace_all(input),
            output
        );

        Ok(())
    }

    #[test]
    fn test_replace_alphabet_list() -> TestResult {
        let list = ListItemReplacer::new()?;
        assert_eq!(
            list.replace_alphabet_list("a. ffegnog b. fgegkl c.", "b"),
            "a. ffegnog \rb∯ fgegkl c."
        );
        Ok(())
    }

    #[test]
    fn test_replace_alphabet_list_parens() -> TestResult {
        let list = ListItemReplacer::new()?;
        assert_eq!(
            list.replace_alphabet_list_parens("a) ffegnog (b) fgegkl c)", "a"),
            "\ra) ffegnog (b) fgegkl c)"
        );
        assert_eq!(
            list.replace_alphabet_list_parens("a) ffegnog (b) fgegkl c)", "b"),
            "a) ffegnog \r&✂&b) fgegkl c)"
        );
        Ok(())
    }

    #[test]
    fn test_iterate_alphabet_array() -> TestResult {
        // NOTE: 이 테스트케이스를 보면 버그때문에 match가 엉터리로 이뤄지고있는것을 볼 수 있지만,
        // pySBD와 동작을 맞추는것이 목표이기때문에 버그도 그대로 유지한다.

        let list = ListItemReplacer::new()?;
        assert_eq!(
            list.iterate_alphabet_array("i. Hi", &list.alphabetical_list_with_periods, false, true),
            "i. Hi"
        );

        let input = "\
Replace

a. Lorem
b. Donec
c. Aenean

Don't

A. Vestibulum
B. Proin
C. Maecenas
";
        let output = "\
Replace

\ra∯ Lorem
\rb∯ Donec
\rc∯ Aenean

Don't

A. Vestibulum
B. Proin
C. Maecenas
";
        assert_eq!(
            list.iterate_alphabet_array(input, &list.alphabetical_list_with_periods, false, false),
            output,
        );

        let input = "\
Do

a) Lorem
b) Donec
c) Aenean

(a) Lorem
(b) Donec
(c) Aenean

Don't

A) Vestibulum
B) Proin
C) Maecenas

(A) Vestibulum
(B) Proin
(C) Maecenas
";
        let output = "\
Do

\r\ra) Lorem
\r\rb) Donec
\r\rc) Aenean

\r&✂&a) Lorem
\r&✂&b) Donec
\r&✂&c) Aenean

Don't

A) Vestibulum
B) Proin
C) Maecenas

(A) Vestibulum
(B) Proin
(C) Maecenas
";
        assert_eq!(
            list.iterate_alphabet_array(input, &list.alphabetical_list_with_parens, true, false),
            output,
        );

        let input = "\
NOP

i. Ut eu volutpat felis.
ii. Mauris
iii. Proin

I. Suspendisse
II. Maecenas
III. Nam
";
        assert_eq!(
            list.iterate_alphabet_array(input, &list.alphabetical_list_with_periods, false, true),
            input,
        );

        let input = "\
Do

i) Ut eu volutpat felis.
ii) Mauris
iii) Proin

(i) Ut eu volutpat felis.
(ii) Mauris
(iii) Proin

Don't

I) Suspendisse
II) Maecenas
III) Nam

(I) Suspendisse
(II) Maecenas
(III) Nam
";
        let output = "\
Do

\r\ri) Ut eu volutpat felis.
\r\rii) Mauris
\r\riii) Proin

\r&✂&i) Ut eu volutpat felis.
\r&✂&ii) Mauris
\r&✂&iii) Proin

Don't

I) Suspendisse
II) Maecenas
III) Nam

(I) Suspendisse
(II) Maecenas
(III) Nam
";
        assert_eq!(
            list.iterate_alphabet_array(input, &list.alphabetical_list_with_parens, true, true),
            output,
        );

        Ok(())
    }

    #[test]
    fn test_scan_lists() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "\
Match below

1.  abcd
2.  xyz
    1. as
    2. yo
3.  asdf
4.  asdf

Dont match below

1.abc
2) asdf
333. asdf
";
        let output = "\
Match below

1♨  abcd
2♨  xyz
    1♨ as
    2♨ yo
3♨  asdf
4♨  asdf

Dont match below

1.abc
2) asdf
333. asdf
";
        assert_eq!(
            list.scan_lists(
                input,
                &list.numbered_list_regex_1,
                &list.numbered_list_regex_2,
                '♨',
                true
            )?,
            Cow::<str>::Borrowed(output)
        );

        let input = "\
1) a
2) b
    1) b1
    2) b2
3) c
4) 5)
55) d
666) e
f77) f
8888) f
10)nomatch
-10) ignore sign
";
        let output = "\
1☝) a
2☝) b
    1☝) b1
    2☝) b2
3☝) c
4☝) 5☝)
55) d
666) e
f77) f
8888) f
10)nomatch
-10) ignore sign
";
        assert_eq!(
            list.scan_lists(
                input,
                &list.numbered_list_parens_regex,
                &list.numbered_list_parens_regex,
                '☝',
                false
            )?,
            Cow::<str>::Borrowed(output)
        );

        Ok(())
    }

    #[test]
    fn test_add_line_breaks_for_numbered_list_with_periods() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "1♨ abcd 2♨ xyz 3♨ asdf 4♨ asdf";
        let output = "1♨ abcd\r2♨ xyz\r3♨ asdf\r4♨ asdf";

        assert_eq!(
            list.add_line_breaks_for_numbered_list_with_periods(input),
            output
        );

        Ok(())
    }

    #[test]
    fn test_add_line_breaks_for_numbered_list_with_parens() -> TestResult {
        let list = ListItemReplacer::new()?;

        let input = "1☝) The first item 2☝) The second item";
        let output = "1☝) The first item\r2☝) The second item";

        assert_eq!(
            list.add_line_breaks_for_numbered_list_with_parens(input),
            output
        );

        Ok(())
    }
}
