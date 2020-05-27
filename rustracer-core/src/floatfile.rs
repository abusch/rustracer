use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Result;
use log::warn;

pub fn read_float_file<P: AsRef<Path>>(filename: P) -> Result<Vec<f32>> {
    let mut values = Vec::new();

    let file = fs::File::open(filename.as_ref())?;
    let reader = BufReader::new(file);

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        for token in line.split_whitespace() {
            match token.parse::<f32>() {
                Ok(float) => values.push(float),
                Err(_) => {
                    warn!(
                        "Unexpected text found at line {} of float file \"{}\"",
                        line_num,
                        filename.as_ref().to_string_lossy()
                    );
                    continue;
                }
            }
        }
    }

    Ok(values)
}
