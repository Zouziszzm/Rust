use std::path::Path;

use crate::config::FilenamePattern;

pub fn matches(path: &Path, pattern: &FilenamePattern) -> bool {
    match pattern {
        FilenamePattern::Glob(glob) => {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            glob.is_match(path) || glob.is_match(file_name)
        }
        FilenamePattern::Regex(regex) => {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            regex.is_match(file_name) || regex.is_match(&path.to_string_lossy())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FilenamePattern;

    fn glob_matcher(pattern: &str) -> FilenamePattern {
        FilenamePattern::Glob(
            globset::GlobBuilder::new(pattern)
                .build()
                .expect("valid glob")
                .compile_matcher(),
        )
    }

    #[test]
    fn glob_matches_rs_files() {
        let pattern = glob_matcher("*.rs");
        assert!(matches(Path::new("main.rs"), &pattern));
        assert!(!matches(Path::new("main.txt"), &pattern));
    }

    #[test]
    fn glob_matches_path() {
        let pattern = glob_matcher("**/*.rs");
        assert!(matches(Path::new("src/lib.rs"), &pattern));
    }
}
