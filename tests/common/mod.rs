//! Shared harness for running tracing fixtures in a child process.
//!
//! Installing a global `log` backend is a one-shot, process-wide operation, and the filters it is
//! built with are frozen at that point. A test that needs a particular `TOPO_LOG` therefore cannot
//! share a process with any other such test. Each one instead runs as a *fixture*: a `#[test]`
//! that no-ops unless a marker environment variable is set, re-executed by the parent through the
//! test binary itself so that its stderr can be captured and asserted on.

#![allow(dead_code)]

use std::process::Command;

/// Set by the parent when re-executing the test binary to run a fixture.
pub const FIXTURE_ENV: &str = "TOPO_FIXTURE";

/// Returns `true` when the current process is *not* running as a fixture child, i.e. when a
/// fixture body should return without doing anything.
///
/// ```ignore
/// #[test]
/// fn fixture_something()
/// {
///     if common::skip_fixture() { return; }
///     // ... real work, output asserted on by the parent test ...
/// }
/// ```
pub fn skip_fixture() -> bool {
    std::env::var_os(FIXTURE_ENV).is_none()
}

/// Runs `fixture` in a child process with `TOPO_LOG` set to `topo_log`, and returns its stderr.
///
/// Panics with the child's full output if it does not exit successfully, so a failing assertion
/// inside the fixture surfaces as a readable failure in the parent.
pub fn run_fixture(
    fixture: &str,
    topo_log: &str,
) -> String {
    let output = Command::new(std::env::current_exe().expect("test binary path"))
        .arg("--exact")
        .arg(fixture)
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(FIXTURE_ENV, "1")
        .env("TOPO_LOG", topo_log)
        .env("NO_COLOR", "1")
        .output()
        .expect("re-executing the test binary");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "fixture `{fixture}` failed with TOPO_LOG={topo_log}\n--- stdout ---\n{}\n--- stderr ---\n{stderr}",
        String::from_utf8_lossy(&output.stdout),
    );
    stderr
}

/// One parsed log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub level: String,
    pub thread: String,
    pub file: String,
    pub line: u32,
    /// Indentation depth in levels, not spaces.
    pub indent: usize,
    pub message: String,
}

/// Column at which the message body starts; must match `MESSAGE_COLUMN` in the library.
const MESSAGE_COLUMN: usize = 40;
/// Spaces per indentation level; must match `INDENT_WIDTH` in the library.
const INDENT_WIDTH: usize = 4;

/// Parses every line of `stderr` that this crate emitted, ignoring anything else the test harness
/// wrote.
///
/// Parsing rather than substring-matching means the assertions also cover the layout itself: a
/// line whose padding or indentation is wrong fails to yield the expected `indent`.
pub fn parse_lines(stderr: &str) -> Vec<LogLine> {
    stderr.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<LogLine> {
    let rest = line.strip_prefix('[')?;
    let close = rest.find(']')?;
    let (prefix, body) = rest.split_at(close);
    let body = &body[1..];

    let open_paren = prefix.find('(')?;
    let close_paren = prefix.find(')')?;
    let level = prefix[..open_paren].trim_end().to_string();
    let thread = prefix[open_paren + 1..close_paren].trim().to_string();

    let location = prefix[close_paren + 1..].trim();
    let (file, line_no) = location.rsplit_once(':')?;
    let line_no: u32 = line_no.parse().ok()?;

    // Everything up to the message is padding-to-column plus indentation.
    let spaces = body.len() - body.trim_start_matches(' ').len();
    let padding = MESSAGE_COLUMN.saturating_sub(location.len()).max(1);
    let indent = spaces.checked_sub(padding)? / INDENT_WIDTH;

    Some(LogLine {
        level,
        thread,
        file: file.to_string(),
        line: line_no,
        indent,
        message: body.trim_start_matches(' ').to_string(),
    })
}

/// Finds the single line whose message is exactly `message`, panicking otherwise.
pub fn line_with<'a>(
    lines: &'a [LogLine],
    message: &str,
) -> &'a LogLine {
    let matches: Vec<&LogLine> = lines.iter().filter(|l| l.message == message).collect();
    assert_eq!(
        matches.len(),
        1,
        "expected exactly one line with message {message:?}, found {}: {lines:#?}",
        matches.len()
    );
    matches[0]
}

/// All messages, in order.
pub fn messages(lines: &[LogLine]) -> Vec<&str> {
    lines.iter().map(|l| l.message.as_str()).collect()
}
