use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::from_str;
use xz2::read::XzDecoder;

use pragmatic_segmenter::Segmenter;

fn main() -> Result<(), Box<dyn Error>> {
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
    let count = dataset.len();
    let bar = ProgressBar::new(count as u64);
    bar.set_draw_delta(10);
    bar.set_style(ProgressStyle::default_bar().template(
        "
{percent}% {wide_bar} {pos:>5}/{len}
{elapsed} passed, Currrent speed: {per_sec}, {eta} left
",
    ));

    let good = dataset
        .par_iter()
        .map(|(input, expected)| {
            bar.inc(1);
            let actual: Vec<_> = segmenter.segment(&input).collect();
            actual == *expected
        })
        .filter(|&b| b)
        .count();
    bar.finish();

    assert_eq!(
        count,
        good,
        "Total={}, Good={}, Bad={}, ({:.3}%)",
        count,
        good,
        count - good,
        good as f64 / count as f64 * 100.0
    );

    Ok(())
}
