/// Return every line in `contents` that contains `pattern`, in order.
///
/// Concepts: iterators (`str::lines()`, `.filter()`/`.collect()` or a plain
/// `for` loop — try both, see which reads better to you), `&str` slices,
/// lifetimes (notice `contents` and the return type share the `'a` lifetime
/// — the lines you return borrow directly from `contents`; nothing gets
/// copied).
pub fn search<'a>(pattern: &str, contents: &'a str) -> Vec<&'a str> {
    // TODO: iterate over contents.lines(), keep only the lines that
    // contain `pattern` (see `str::contains`), and collect them into a Vec.
    todo!("find every line in `contents` containing `pattern`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matching_lines() {
        let contents = "\
first line
second line with error
third line
fourth line: error again";

        let results = search("error", contents);

        assert_eq!(
            results,
            vec!["second line with error", "fourth line: error again"]
        );
    }

    #[test]
    fn returns_empty_vec_when_nothing_matches() {
        let contents = "nothing to see here\njust normal lines";
        assert!(search("error", contents).is_empty());
    }

    #[test]
    fn is_case_sensitive_by_default() {
        let contents = "ERROR in caps\nerror in lowercase";
        assert_eq!(search("error", contents), vec!["error in lowercase"]);
    }
}
