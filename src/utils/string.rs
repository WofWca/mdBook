use itertools::Itertools;
use regex::Regex;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::RangeBounds;

/// Take a range of lines from a string.
pub fn take_lines<R: RangeBounds<usize>>(s: &str, range: R) -> String {
    let start = match range.start_bound() {
        Excluded(&n) => n + 1,
        Included(&n) => n,
        Unbounded => 0,
    };
    let mut lines = s.lines().skip(start);
    match range.end_bound() {
        Excluded(end) => lines.take(end.saturating_sub(start)).join("\n"),
        Included(end) => lines.take((end + 1).saturating_sub(start)).join("\n"),
        Unbounded => lines.join("\n"),
    }
}

/// Take anchored lines from a string.
/// Lines containing anchor are ignored.
pub fn take_anchored_lines(s: &str, anchor: &str) -> String {
    lazy_static! {
        static ref RE_START: Regex = Regex::new(r"ANCHOR:\s*(?P<anchor_name>[\w_-]+)").unwrap();
        static ref RE_END: Regex = Regex::new(r"ANCHOR_END:\s*(?P<anchor_name>[\w_-]+)").unwrap();
    }

    let mut retained = Vec::<&str>::new();
    let mut anchor_found = false;

    for l in s.lines() {
        if anchor_found {
            match RE_END.captures(l) {
                Some(cap) => {
                    if &cap["anchor_name"] == anchor {
                        break;
                    }
                }
                None => {
                    if !RE_START.is_match(l) {
                        retained.push(l);
                    }
                }
            }
        } else {
            if let Some(cap) = RE_START.captures(l) {
                if &cap["anchor_name"] == anchor {
                    anchor_found = true;
                }
            }
        }
    }

    retained.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{take_anchored_lines, take_lines};

    #[test]
    fn take_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(take_lines(s, 1..3), "ipsum\ndolor");
        assert_eq!(take_lines(s, 3..), "sit\namet");
        assert_eq!(take_lines(s, ..3), "Lorem\nipsum\ndolor");
        assert_eq!(take_lines(s, ..), s);
        // corner cases
        assert_eq!(take_lines(s, 4..3), "");
        assert_eq!(take_lines(s, ..100), s);
    }

    #[test]
    fn take_anchored_lines_test() {
        let s = "Lorem\nipsum\ndolor\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "");

        let s = "Lorem\nipsum\ndolor\nANCHOR_END: test\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "");

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet";
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nANCHOR: test\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nipsum";
        assert_eq!(take_anchored_lines(s, "test"), "ipsum\ndolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");

        let s = "Lorem\nANCHOR:    test2\nipsum\nANCHOR: test\ndolor\nsit\namet\nANCHOR_END: test\nlorem\nANCHOR_END:test2\nipsum";
        assert_eq!(
            take_anchored_lines(s, "test2"),
            "ipsum\ndolor\nsit\namet\nlorem"
        );
        assert_eq!(take_anchored_lines(s, "test"), "dolor\nsit\namet");
        assert_eq!(take_anchored_lines(s, "something"), "");
    }
}
