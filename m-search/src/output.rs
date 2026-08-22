use std::io::{self, Write};

use crate::SearchResult;
use serde_json;

pub fn print_results(results: &[SearchResult], json: bool) -> io::Result<()> {
    let mut stdout = io::stdout().lock();

    if json {
        serde_json::to_writer_pretty(&mut stdout, results)?;
        writeln!(stdout)?;
        return Ok(());
    }

    for result in results {
        if result.content_matches.is_empty() {
            writeln!(stdout, "{}", result.path.display())?;
            continue;
        }

        for line_match in &result.content_matches {
            writeln!(
                stdout,
                "{}:{}:{}",
                result.path.display(),
                line_match.line_number,
                line_match.line
            )?;
        }
    }

    Ok(())
}
