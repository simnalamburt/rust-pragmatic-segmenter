use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;

use onig::{Captures, Regex, RegexOptions, Syntax};

struct Segmenter {
    roman_numerals: HashMap<&'static str, isize>,
    latin_numerals: HashMap<&'static str, isize>,

    alphabetical_list_with_periods: Regex,
    alphabetical_list_with_parens: Regex,

    alphabetical_list_letters_and_periods_regex: Regex,
    extract_alphabetical_list_letters_regex: Regex,
}

// TODO: 에러 핸들링 바르게 하기
type SegmenterResult<T> = Result<T, Box<dyn Error>>;

impl Segmenter {
    fn new() -> SegmenterResult<Self> {
        Ok(Segmenter {
            roman_numerals: [
                "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii",
                "xiv", "x", "xi", "xii", "xiii", "xv", "xvi", "xvii", "xviii", "xix", "xx",
            ]
            .iter()
            .enumerate()
            .map(|(idx, s)| (*s, idx as isize))
            .collect(),

            latin_numerals: [
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ]
            .iter()
            .enumerate()
            .map(|(idx, s)| (*s, idx as isize))
            .collect(),

            // Example: https://rubular.com/r/XcpaJKH0sz
            //
            // NOTE: 루비 버전은 case sensitive하고, 파이썬 버전은 case insensitive한데, 루비
            // 버전에서 case sensitive하게 만들어진것이 실수같음. Case insensitive하게 만든다.
            alphabetical_list_with_periods: Regex::with_options(
                r"(?<=^)[a-z](?=\.)|(?<=\A)[a-z](?=\.)|(?<=\s)[a-z](?=\.)",
                RegexOptions::REGEX_OPTION_IGNORECASE,
                Syntax::ruby(),
            )?,

            // Example: https://rubular.com/r/Gu5rQapywf
            alphabetical_list_with_parens: Regex::with_options(
                r"(?<=\()[a-z]+(?=\))|(?<=^)[a-z]+(?=\))|(?<=\A)[a-z]+(?=\))|(?<=\s)[a-z]+(?=\))",
                RegexOptions::REGEX_OPTION_IGNORECASE,
                Syntax::ruby(),
            )?,

            // Example: https://rubular.com/r/wMpnVedEIb
            alphabetical_list_letters_and_periods_regex: Regex::with_options(
                r"(?<=^)[a-z]\.|(?<=\A)[a-z]\.|(?<=\s)[a-z]\.",
                RegexOptions::REGEX_OPTION_IGNORECASE,
                Syntax::ruby(),
            )?,

            // Example: https://rubular.com/r/NsNFSqrNvJ
            extract_alphabetical_list_letters_regex: Regex::with_options(
                r"\([a-z]+(?=\))|(?<=^)[a-z]+(?=\))|(?<=\A)[a-z]+(?=\))|(?<=\s)[a-z]+(?=\))",
                RegexOptions::REGEX_OPTION_IGNORECASE,
                Syntax::ruby(),
            )?,
        })
    }
}

#[cfg(test)]
type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn test_alphabetical_list_with_periods() -> TestResult {
    let seg = Segmenter::new()?;
    let text = "a. The first item b. The second item c. The third list item D. case insesitive \
E. Don't select the nextF.dont't select this G should be followed by dot";

    assert_eq!(
        seg.alphabetical_list_with_periods
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
    let seg = Segmenter::new()?;
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
        seg.alphabetical_list_with_parens
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
    let seg = Segmenter::new()?;
    let text = "His name is Mark E. Smith. a. here it is b. another c. one more
 They went to the store. It was John A. Smith. She was Jane B. Smith.";

    assert_eq!(
        seg.alphabetical_list_letters_and_periods_regex
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
    let seg = Segmenter::new()?;
    let text =
        "a) here it is b) another c) one more \nThey went to the store. W) hello X) hello Y) hello";

    assert_eq!(
        seg.extract_alphabetical_list_letters_regex
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

impl Segmenter {
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
}

#[test]
fn test_replace_alphabet_list() -> TestResult {
    let seg = Segmenter::new()?;
    assert_eq!(
        seg.replace_alphabet_list("a. ffegnog b. fgegkl c.", "b"),
        "a. ffegnog \rb∯ fgegkl c."
    );
    Ok(())
}

impl Segmenter {
    fn replace_alphabet_list_parens(&self, text: &str, what_to_replace: &str) -> String {
        self.extract_alphabetical_list_letters_regex
            .replace_all(text, |m: &Captures| {
                let mat = m.at(0).unwrap(); // Must exists

                // TODO: 루비코드에선 검사하기 전에 mat을 downcase 한다. 파이썬에선 안함. downcase
                // 하는것이 맞지만, 일단은 pySBD와 같은 동작을 만들겠다.
                //
                // Reference:
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c81f9d1d7fb3abd4c1e2e266fa5b34c42/lib/pragmatic_segmenter/list.rb#L149
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
}

#[test]
fn test_replace_alphabet_list_parens() -> TestResult {
    let seg = Segmenter::new()?;
    assert_eq!(
        seg.replace_alphabet_list_parens("a) ffegnog (b) fgegkl c)", "a"),
        "\ra) ffegnog (b) fgegkl c)"
    );
    assert_eq!(
        seg.replace_alphabet_list_parens("a) ffegnog (b) fgegkl c)", "b"),
        "a) ffegnog \r&✂&b) fgegkl c)"
    );
    Ok(())
}

impl Segmenter {
    fn iterate_alphabet_array<'a>(
        &self,
        text: &'a str,
        regex: &Regex,
        parens: bool,
        use_roman_numeral: bool,
    ) -> Cow<'a, str> {
        // TODO: 루비 코드(pragmatic segmenter)에선 여기서 검사하기 전에 downcase를 함, pySBD에선
        // 안함. Downcase를 하는것이 맞지만, 이 프로젝트는 일단 pySBD의 동작을 따르겠다.
        //
        // Reference:
        //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c81f9d1d7fb3abd4c1e2e266fa5b34c42/lib/pragmatic_segmenter/list.rb#L186
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

        // TODO: 이하 코드에서 매 루프마다 복사가 발생함, 최적화 가능함
        let mut result: Cow<str> = Cow::Borrowed(text);
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
                //   https://github.com/diasks2/pragmatic_segmenter/blob/1ade491c81f9d1d7fb3abd4c1e2e266fa5b34c42/lib/pragmatic_segmenter/list.rb#L194
                //   https://github.com/nipunsadvilkar/pySBD/blob/90699972c8b5cd63c7fa4581419250e60b15db87/pysbd/lists_item_replacer.py#L235
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
}

#[test]
fn test_iterate_alphabet_array() -> TestResult {
    // TODO: 이 테스트케이스를 보면 버그때문에 match가 엉터리로 이뤄지고있는것을 볼 수 있지만,
    // pySBD와 동작을 맞추는것이 목표이기때문에 버그도 그대로 유지한다.

    let seg = Segmenter::new()?;
    assert_eq!(
        seg.iterate_alphabet_array("i. Hi", &seg.alphabetical_list_with_periods, false, true),
        String::from("i. Hi")
    );

    //
    // Test large inputs
    //
    let text = "\
List 1

a. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b. Donec interdum lectus sed facilisis accumsan.
c. Aenean aliquam suscipit elit quis lobortis.

A. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B. Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C. Maecenas hendrerit eros nisi.

i. Ut eu volutpat felis.
ii. Mauris varius felis sed scelerisque auctor.
iii. Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I. Suspendisse placerat neque non leo aliquam pharetra.
II. Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III. Nam vestibulum magna id lectus tristique egestas.

List 2

a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b) Donec interdum lectus sed facilisis accumsan.
c) Aenean aliquam suscipit elit quis lobortis.

A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C) Maecenas hendrerit eros nisi.

i) Ut eu volutpat felis.
ii) Mauris varius felis sed scelerisque auctor.
iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I) Suspendisse placerat neque non leo aliquam pharetra.
II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III) Nam vestibulum magna id lectus tristique egestas.

List 3

(a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
(b) Donec interdum lectus sed facilisis accumsan.
(c) Aenean aliquam suscipit elit quis lobortis.

(A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
(B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
(C) Maecenas hendrerit eros nisi.

(i) Ut eu volutpat felis.
(ii) Mauris varius felis sed scelerisque auctor.
(iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

(I) Suspendisse placerat neque non leo aliquam pharetra.
(II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
(III) Nam vestibulum magna id lectus tristique egestas.
";

    assert_eq!(
        seg.iterate_alphabet_array(text, &seg.alphabetical_list_with_periods, false, false),
        String::from("\
List 1

\ra∯ Lorem ipsum dolor sit amet, consectetur adipiscing elit.
\rb∯ Donec interdum lectus sed facilisis accumsan.
\rc∯ Aenean aliquam suscipit elit quis lobortis.

A. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B. Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C. Maecenas hendrerit eros nisi.

i. Ut eu volutpat felis.
ii. Mauris varius felis sed scelerisque auctor.
iii. Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I. Suspendisse placerat neque non leo aliquam pharetra.
II. Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III. Nam vestibulum magna id lectus tristique egestas.

List 2

a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b) Donec interdum lectus sed facilisis accumsan.
c) Aenean aliquam suscipit elit quis lobortis.

A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C) Maecenas hendrerit eros nisi.

i) Ut eu volutpat felis.
ii) Mauris varius felis sed scelerisque auctor.
iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I) Suspendisse placerat neque non leo aliquam pharetra.
II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III) Nam vestibulum magna id lectus tristique egestas.

List 3

(a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
(b) Donec interdum lectus sed facilisis accumsan.
(c) Aenean aliquam suscipit elit quis lobortis.

(A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
(B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
(C) Maecenas hendrerit eros nisi.

(i) Ut eu volutpat felis.
(ii) Mauris varius felis sed scelerisque auctor.
(iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

(I) Suspendisse placerat neque non leo aliquam pharetra.
(II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
(III) Nam vestibulum magna id lectus tristique egestas.
"),
    );
    assert_eq!(
        seg.iterate_alphabet_array(text, &seg.alphabetical_list_with_parens,  true,  false),
        String::from("\
List 1

a. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b. Donec interdum lectus sed facilisis accumsan.
c. Aenean aliquam suscipit elit quis lobortis.

A. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B. Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C. Maecenas hendrerit eros nisi.

i. Ut eu volutpat felis.
ii. Mauris varius felis sed scelerisque auctor.
iii. Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I. Suspendisse placerat neque non leo aliquam pharetra.
II. Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III. Nam vestibulum magna id lectus tristique egestas.

List 2

\r\ra) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
\r\rb) Donec interdum lectus sed facilisis accumsan.
\r\rc) Aenean aliquam suscipit elit quis lobortis.

A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C) Maecenas hendrerit eros nisi.

i) Ut eu volutpat felis.
ii) Mauris varius felis sed scelerisque auctor.
iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I) Suspendisse placerat neque non leo aliquam pharetra.
II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III) Nam vestibulum magna id lectus tristique egestas.

List 3

\r&✂&a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
\r&✂&b) Donec interdum lectus sed facilisis accumsan.
\r&✂&c) Aenean aliquam suscipit elit quis lobortis.

(A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
(B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
(C) Maecenas hendrerit eros nisi.

(i) Ut eu volutpat felis.
(ii) Mauris varius felis sed scelerisque auctor.
(iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

(I) Suspendisse placerat neque non leo aliquam pharetra.
(II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
(III) Nam vestibulum magna id lectus tristique egestas.
"),
    );
    assert_eq!(
        seg.iterate_alphabet_array(text, &seg.alphabetical_list_with_periods, false, true),
        String::from("\
List 1

a. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b. Donec interdum lectus sed facilisis accumsan.
c. Aenean aliquam suscipit elit quis lobortis.

A. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B. Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C. Maecenas hendrerit eros nisi.

i. Ut eu volutpat felis.
ii. Mauris varius felis sed scelerisque auctor.
iii. Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I. Suspendisse placerat neque non leo aliquam pharetra.
II. Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III. Nam vestibulum magna id lectus tristique egestas.

List 2

a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b) Donec interdum lectus sed facilisis accumsan.
c) Aenean aliquam suscipit elit quis lobortis.

A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C) Maecenas hendrerit eros nisi.

i) Ut eu volutpat felis.
ii) Mauris varius felis sed scelerisque auctor.
iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I) Suspendisse placerat neque non leo aliquam pharetra.
II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III) Nam vestibulum magna id lectus tristique egestas.

List 3

(a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
(b) Donec interdum lectus sed facilisis accumsan.
(c) Aenean aliquam suscipit elit quis lobortis.

(A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
(B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
(C) Maecenas hendrerit eros nisi.

(i) Ut eu volutpat felis.
(ii) Mauris varius felis sed scelerisque auctor.
(iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

(I) Suspendisse placerat neque non leo aliquam pharetra.
(II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
(III) Nam vestibulum magna id lectus tristique egestas.
"),
    );
    assert_eq!(
        seg.iterate_alphabet_array(text, &seg.alphabetical_list_with_parens,  true,  true),
        String::from("\
List 1

a. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b. Donec interdum lectus sed facilisis accumsan.
c. Aenean aliquam suscipit elit quis lobortis.

A. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B. Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C. Maecenas hendrerit eros nisi.

i. Ut eu volutpat felis.
ii. Mauris varius felis sed scelerisque auctor.
iii. Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I. Suspendisse placerat neque non leo aliquam pharetra.
II. Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III. Nam vestibulum magna id lectus tristique egestas.

List 2

a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
b) Donec interdum lectus sed facilisis accumsan.
c) Aenean aliquam suscipit elit quis lobortis.

A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
C) Maecenas hendrerit eros nisi.

\r\ri) Ut eu volutpat felis.
\r\rii) Mauris varius felis sed scelerisque auctor.
\r\riii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

I) Suspendisse placerat neque non leo aliquam pharetra.
II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
III) Nam vestibulum magna id lectus tristique egestas.

List 3

(a) Lorem ipsum dolor sit amet, consectetur adipiscing elit.
(b) Donec interdum lectus sed facilisis accumsan.
(c) Aenean aliquam suscipit elit quis lobortis.

(A) Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Suspendisse ligula nulla, interdum at gravida tempor, pulvinar at nunc.
(B) Proin porta, tellus sit amet condimentum scelerisque, orci urna gravida libero, at semper lectus felis a metus.
(C) Maecenas hendrerit eros nisi.

\r&✂&i) Ut eu volutpat felis.
\r&✂&ii) Mauris varius felis sed scelerisque auctor.
\r&✂&iii) Proin leo nunc, pretium et rhoncus sed, hendrerit id ipsum.

(I) Suspendisse placerat neque non leo aliquam pharetra.
(II) Maecenas tempor auctor mauris, non ullamcorper dui posuere nec.
(III) Nam vestibulum magna id lectus tristique egestas.
"),
    );

    Ok(())
}
