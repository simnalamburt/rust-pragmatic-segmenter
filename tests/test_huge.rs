use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use rayon::prelude::*;
use serde_json::from_str;
use xz2::read::XzDecoder;

use pragmatic_segmenter::Segmenter;

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug)]
struct DifferentFromPySBD {
    expected: Vec<String>,
    actual: Vec<String>,
}

impl Display for DifferentFromPySBD {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DifferntFromPySBD {{ pySBD's result is {:#?}, but ours is {:#?} }}",
            self.expected, self.actual
        )
    }
}

impl Error for DifferentFromPySBD {}

#[test]
fn test_all() -> TestResult {
    let segmenter = Segmenter::new()?;

    let inputs = BufReader::new(XzDecoder::new(File::open("tests/fixtures/inputs.xz")?));
    let outputs = BufReader::new(XzDecoder::new(File::open("tests/fixtures/outputs.xz")?));
    let dataset: Vec<_> = inputs
        .lines()
        .zip(outputs.lines())
        .map(|(input, output)| {
            let input: String = from_str(&input?)?;
            let output: Vec<String> = from_str(&output?)?;

            Ok((input, output))
        })
        .collect::<io::Result<_>>()?;

    dataset
        .par_iter()
        .map(|(input, expected)| -> Result<(), _> {
            let actual: Vec<_> = segmenter.segment(&input).collect();
            if actual == *expected {
                Ok(())
            } else {
                Err(DifferentFromPySBD {
                    expected: expected.clone(),
                    actual: actual.into_iter().map(|s| s.to_string()).collect(),
                })
            }
        })
        .collect::<Result<(), _>>()?;

    Ok(())
}
