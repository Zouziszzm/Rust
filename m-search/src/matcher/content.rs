use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use regex::Regex;

const BINARY_CHECK_BYTES: usize = 8_000;

pub fn search_file(path: &Path, regex: &Regex) -> Option<Vec<crate::LineMatch>> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    if is_binary(&mut file) {
        return None;
    }

    file.seek(SeekFrom::Start(0)).ok()?;

    let reader = BufReader::new(file);
    let mut matches = Vec::new();

    for (idx, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => return None,
        };

        if regex.is_match(&line) {
            matches.push(crate::LineMatch {
                line_number: idx + 1,
                line,
            });
        }
    }

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

fn is_binary(file: &mut File) -> bool {
    let mut buf = [0u8; BINARY_CHECK_BYTES];
    let n = match file.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return true,
    };

    buf[..n].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn finds_matching_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "hello world").unwrap();
        writeln!(file, "fn main() {{}}").unwrap();
        writeln!(file, "goodbye").unwrap();

        let regex = Regex::new(r"fn main").unwrap();
        let matches = search_file(file.path(), &regex).unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line_number, 2);
        assert!(matches[0].line.contains("fn main"));
    }

    #[test]
    fn skips_binary_files() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0u8, 1, 2, 3]).unwrap();

        let regex = Regex::new(".*").unwrap();
        assert!(search_file(file.path(), &regex).is_none());
    }
}
