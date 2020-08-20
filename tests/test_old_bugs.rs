use std::error::Error;

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
