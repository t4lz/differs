// Copied and adapted from https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs
use git2::{DiffDelta, DiffHunk, DiffLine};

const RESET: &str = "\u{1b}[m";
const BOLD: &str = "\u{1b}[1m";
const RED: &str = "\u{1b}[31m";
const GREEN: &str = "\u{1b}[32m";
const CYAN: &str = "\u{1b}[36m";

fn line_color(line: &DiffLine) -> Option<&'static str> {
    match line.origin() {
        '+' => Some(GREEN),
        '-' => Some(RED),
        '>' => Some(GREEN),
        '<' => Some(RED),
        'F' => Some(BOLD),
        'H' => Some(CYAN),
        _ => None,
    }
}

pub(crate) fn print_diff_line(
    _delta: DiffDelta,
    _hunk: Option<DiffHunk>,
    line: DiffLine,
) -> bool {
    print!("{}", RESET);
    if let Some(color) = line_color(&line) {
        print!("{}", color);
    }
    match line.origin() {
        '+' | '-' | ' ' => print!("{}", line.origin()),
        _ => {}
    }
    print!("{}", str::from_utf8(line.content()).unwrap());
    true
}
