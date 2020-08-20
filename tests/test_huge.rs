use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use xz2::read::XzDecoder;

use pragmatic_segmenter::Segmenter;

type TestResult = Result<(), Box<dyn Error>>;

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
