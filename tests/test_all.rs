use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use xz2::read::XzDecoder;

use pragmatic_segmenter::Segmenter;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn test_punctuation() -> TestResult {
    let segmenter = Segmenter::new()?;

    assert_eq!(
        segmenter
            .segment("U.S. army at www.stanler.com")
            .collect::<Vec<_>>(),
        vec!["U.S. army at www.stanler.com"],
    );

    Ok(())
}

#[test]
fn test_character_boundary() -> TestResult {
    let segmenter = Segmenter::new()?;
    let input = "U.S. and NYSE’s U.S.";

    let actual: Vec<_> = segmenter.segment(input).collect();
    let expected = vec!["U.S. and NYSE’s U.S."];

    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_parens() -> TestResult {
    let segmenter = Segmenter::new()?;
    let input = "AA Inc. is including";

    let actual: Vec<_> = segmenter.segment(input).collect();
    let expected = vec!["AA Inc. is including"];

    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_quotes() -> TestResult {
    let segmenter = Segmenter::new()?;

    let input = r#"Our "business." Walgreens"#;
    let actual: Vec<_> = segmenter.segment(input).collect();
    let expected = vec![r#"Our "business." "#, "Walgreens"];

    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn test_all() -> TestResult {
    let segmenter = Segmenter::new()?;

    let inputs = BufReader::new(XzDecoder::new(File::open("tests/fixtures/inputs.xz")?));
    let outputs = BufReader::new(XzDecoder::new(File::open("tests/fixtures/outputs.xz")?));

    let mut bad = 0;
    for each in std::iter::Iterator::zip(inputs.lines(), outputs.lines()).enumerate() {
        let (i, (input, output)) = each;
        let input: String = serde_json::from_str(&input?)?;

        let expected: Vec<String> = serde_json::from_str(&output?)?;
        let actual: Vec<_> = segmenter.segment(&input).collect();

        if actual != expected {
            bad += 1
        }
        if i % 250 == 0 {
            println!(
                "{:5}: Good={:5}, Bad={:5}, {:5}%",
                i,
                i + 1 - bad,
                bad,
                (i + 1 - bad) as f64 / (i + 1) as f64 * 100.0
            );
        }
    }
    assert_eq!(bad, 0);

    Ok(())
}
