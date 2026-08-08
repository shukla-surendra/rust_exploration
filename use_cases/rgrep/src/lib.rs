pub mod cli;
pub mod error;
pub mod search;

use cli::Config;
use error::RgrepError;

/// Run one full `rgrep` invocation: read every file in `config.paths`,
/// search it for `config.pattern`, and print matching lines.
///
/// Concepts: File I/O (`std::fs::read_to_string`), `Result<T, E>` and the
/// `?` operator for propagating errors up to `main`, `match`, and — once
/// you're searching more than one file — deciding whether to prefix each
/// printed line with its filename (real `grep` does this automatically
/// when given multiple files; try matching that behavior).
pub fn run(config: Config) -> Result<(), RgrepError> {
    // TODO, roughly:
    //   for path in &config.paths {
    //       - read the file into a String
    //         (std::fs::read_to_string returns Result<String, io::Error> —
    //          turn a failure into your RgrepError::Io variant, with `path`
    //          attached, instead of just using `?` directly)
    //       - call search::search(&config.pattern, &contents)
    //       - print each matching line (println!)
    //         - if config.paths.len() > 1, prefix each line with
    //           "path: " the way real grep does
    //   }
    todo!("read each path in config.paths, search it, print matches")
}
