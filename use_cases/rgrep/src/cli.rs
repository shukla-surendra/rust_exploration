use crate::error::RgrepError;

/// Parsed command-line arguments for a run of `rgrep`.
///
/// Called like: `rgrep "error" app.log` or `rgrep "error" *.log`
/// (the shell expands `*.log` into multiple arguments before your program
/// ever sees them — that's why `paths` is a `Vec`, not a single `String`.)
///
/// Concepts: structs, `String` vs `&str` (why owned `String` here and not
/// borrowed `&str` — think about how long `args` lives vs how long `Config`
/// needs to live), `Vec<T>`.
pub struct Config {
    pub pattern: String,
    pub paths: Vec<String>,
    // Stretch goal (do this after the base version works): add a field like
    // `pub case_insensitive: bool` and parse a `-i` flag out of `args` below.
}

impl Config {
    /// Build a `Config` from raw process args (including `args[0]`, the
    /// program name — you'll need to skip it).
    ///
    /// Concepts: slices (`&[String]`), ownership and borrowing (do you need
    /// to `.clone()` here, or can you take ownership with `.to_string()` /
    /// indexing + `into()`? think about what `args` looks like to the
    /// caller after this returns), `Result<T, E>`, `match` / `if let`.
    pub fn build(args: &[String]) -> Result<Config, RgrepError> {
        // TODO:
        //   1. args[0] is the program name — ignore it.
        //   2. args[1], if present, is the pattern. If it's missing,
        //      return an appropriate RgrepError.
        //   3. args[2..], if non-empty, are the file paths to search.
        //      If there are none, return an appropriate RgrepError.
        //   4. Build and return `Ok(Config { pattern, paths })`.
        todo!("parse `args` into a Config, or a RgrepError if usage is wrong")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_config_from_valid_args() {
        let args = vec![
            "rgrep".to_string(),
            "error".to_string(),
            "app.log".to_string(),
        ];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.pattern, "error");
        assert_eq!(config.paths, vec!["app.log".to_string()]);
    }

    #[test]
    fn accepts_multiple_paths() {
        let args = vec![
            "rgrep".to_string(),
            "error".to_string(),
            "app.log".to_string(),
            "app2.log".to_string(),
        ];
        let config = Config::build(&args).unwrap();
        assert_eq!(config.paths.len(), 2);
    }

    #[test]
    fn errors_when_pattern_missing() {
        let args = vec!["rgrep".to_string()];
        assert!(Config::build(&args).is_err());
    }

    #[test]
    fn errors_when_paths_missing() {
        let args = vec!["rgrep".to_string(), "error".to_string()];
        assert!(Config::build(&args).is_err());
    }
}
