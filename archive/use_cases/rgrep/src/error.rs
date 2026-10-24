use std::fmt;

/// All the ways `rgrep` can fail, collected into one type so `main` only
/// has to handle one `Result` error type end to end.
///
/// Concepts: enums, `Result<T, E>`, the `std::error::Error` trait, `match`.
#[derive(Debug)]
pub enum RgrepError {
    // TODO: what does a usage error need to say? e.g. no pattern given.
    // MissingPattern,

    // TODO: what does a usage error need to say? e.g. no file paths given.
    // MissingPaths,

    // TODO: reading a file can fail (missing file, permissions, ...).
    // Hint: wrap the path that failed AND the underlying `std::io::Error`,
    // so the message can say *which* file broke.
    // Io { path: String, source: std::io::Error },
}

// `Display` controls what users see when this error is printed with `{}`
// (as opposed to `Debug`'s `{:?}`, which `#[derive(Debug)]` already gave us
// above). This is the trait you write by hand instead of deriving, because
// the message wording is up to you.
impl fmt::Display for RgrepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: match on `self` and `write!(f, "...")` a human-readable
        // message for each variant you defined above.
        todo!("format each RgrepError variant as a human-readable message")
    }
}

// Implementing this (empty is fine) is what lets `RgrepError` be used
// anywhere a `Box<dyn std::error::Error>` or `?` chain expects a "real"
// error type, and is what makes `{e}` work in `eprintln!("{e}")` via Display.
impl std::error::Error for RgrepError {}
