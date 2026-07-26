//! End-to-end tests for the logging macros, levels, indentation and output layout.
//!
//! Every assertion here is made against output actually captured from a child process; see
//! `tests/common/mod.rs` for why the fixtures cannot share one.
//!
//! These assertions need the trace points compiled in; the feature-off build is covered by
//! `tests/feature_switch_test.rs`.
#![cfg(feature = "trace")]

mod common;

use common::{line_with, messages, parse_lines, run_fixture};
use topohedral_tracing::{
    debug, decrement_indent, error, increment_indent, info, init, trace, trace_scope, warn,
};

//{{{ fixtures

#[test]
fn fixture_all_levels() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    trace!("level trace");
    debug!("level debug");
    info!("level info");
    warn!("level warn");
    error!("level error");

    trace!(target: "custom", "targeted trace");
    info!(target: "custom", "targeted info");
}

#[test]
fn fixture_manual_indentation() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    info!("depth 0");
    increment_indent();
    info!("depth 1");
    increment_indent();
    info!("depth 2");
    decrement_indent();
    info!("back to depth 1");
    decrement_indent();
    info!("back to depth 0");
}

#[test]
fn fixture_trace_scope_nesting() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    fn inner() {
        trace_scope!("inner");
        info!("in inner");
    }

    fn outer() {
        trace_scope!("outer");
        info!("in outer");
        inner();
        info!("back in outer");
    }

    info!("before");
    outer();
    info!("after");
}

#[test]
fn fixture_scope_unwinds_on_early_return() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    fn bails() -> Option<()> {
        trace_scope!("bails");
        info!("about to bail");
        None?;
        info!("unreachable");
        Some(())
    }

    assert!(bails().is_none());
    info!("after bailing");
}

#[test]
fn fixture_indentation_is_per_thread() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    increment_indent();
    increment_indent();
    info!("parent at depth 2");

    std::thread::spawn(|| {
        info!("child at depth 0");
    })
    .join()
    .unwrap();
}

#[test]
fn fixture_dependency_log_macros() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    // A dependency that knows nothing about this crate logs through `log` directly. Because we
    // install ourselves as the global backend, it must come out formatted and indented like ours.
    increment_indent();
    log::info!("emitted through the log facade");
}

//}}}
//{{{ tests

#[test]
fn every_level_is_emitted_and_labelled() {
    let lines = parse_lines(&run_fixture("fixture_all_levels", "all=trace"));

    assert_eq!(line_with(&lines, "level trace").level, "TRACE");
    assert_eq!(line_with(&lines, "level debug").level, "DEBUG");
    assert_eq!(line_with(&lines, "level info").level, "INFO");
    assert_eq!(line_with(&lines, "level warn").level, "WARN");
    assert_eq!(line_with(&lines, "level error").level, "ERROR");
}

#[test]
fn lines_carry_the_call_site_not_the_library() {
    let lines = parse_lines(&run_fixture("fixture_all_levels", "all=trace"));
    let line = line_with(&lines, "level info");

    assert_eq!(line.file, "logging_test.rs");
    assert!(line.line > 0, "line number should be recorded: {line:?}");
    assert!(
        lines.iter().all(|l| l.file == "logging_test.rs"),
        "no line should be attributed to the tracing crate: {lines:#?}"
    );
}

#[test]
fn a_level_filter_hides_more_verbose_levels() {
    let lines = parse_lines(&run_fixture("fixture_all_levels", "all=warn"));

    assert_eq!(messages(&lines), vec!["level warn", "level error"]);
}

#[test]
fn explicit_targets_are_filtered_independently() {
    let stderr = run_fixture("fixture_all_levels", "all=off,custom=info");
    let lines = parse_lines(&stderr);

    assert_eq!(
        messages(&lines),
        vec!["targeted info"],
        "only the `custom` target at info or below should survive"
    );
}

#[test]
fn manual_indentation_nests_and_unwinds() {
    let lines = parse_lines(&run_fixture("fixture_manual_indentation", "all=info"));

    assert_eq!(line_with(&lines, "depth 0").indent, 0);
    assert_eq!(line_with(&lines, "depth 1").indent, 1);
    assert_eq!(line_with(&lines, "depth 2").indent, 2);
    assert_eq!(line_with(&lines, "back to depth 1").indent, 1);
    assert_eq!(line_with(&lines, "back to depth 0").indent, 0);
}

#[test]
fn trace_scope_brackets_and_indents_the_scope() {
    let lines = parse_lines(&run_fixture("fixture_trace_scope_nesting", "all=info"));

    assert_eq!(
        messages(&lines),
        vec![
            "before",
            "* Entering outer",
            "in outer",
            "* Entering inner",
            "in inner",
            "* Leaving inner",
            "back in outer",
            "* Leaving outer",
            "after",
        ]
    );

    // Entry and exit sit at the outer level; the scope body sits one level in.
    assert_eq!(line_with(&lines, "* Entering outer").indent, 0);
    assert_eq!(line_with(&lines, "in outer").indent, 1);
    assert_eq!(line_with(&lines, "* Entering inner").indent, 1);
    assert_eq!(line_with(&lines, "in inner").indent, 2);
    assert_eq!(line_with(&lines, "* Leaving inner").indent, 1);
    assert_eq!(line_with(&lines, "* Leaving outer").indent, 0);
    assert_eq!(line_with(&lines, "after").indent, 0);
}

#[test]
fn scope_exit_is_logged_on_early_return() {
    let lines = parse_lines(&run_fixture(
        "fixture_scope_unwinds_on_early_return",
        "all=info",
    ));

    assert_eq!(
        messages(&lines),
        vec![
            "* Entering bails",
            "about to bail",
            "* Leaving bails",
            "after bailing"
        ],
        "`?` must still run the guard's destructor"
    );
    assert_eq!(line_with(&lines, "after bailing").indent, 0);
}

#[test]
fn indentation_does_not_leak_between_threads() {
    let lines = parse_lines(&run_fixture(
        "fixture_indentation_is_per_thread",
        "all=info",
    ));

    let parent = line_with(&lines, "parent at depth 2");
    let child = line_with(&lines, "child at depth 0");

    assert_eq!(parent.indent, 2);
    assert_eq!(child.indent, 0, "a new thread starts unindented");
    assert_ne!(parent.thread, child.thread, "thread ids should differ");
}

#[test]
fn records_from_the_log_facade_are_formatted_by_us() {
    let lines = parse_lines(&run_fixture("fixture_dependency_log_macros", "all=info"));
    let line = line_with(&lines, "emitted through the log facade");

    assert_eq!(line.level, "INFO");
    assert_eq!(line.indent, 1, "facade records share our indentation");
    assert_eq!(line.file, "logging_test.rs");
}

#[test]
fn no_topo_log_means_no_output() {
    let lines = parse_lines(&run_fixture("fixture_all_levels", ""));
    assert!(lines.is_empty(), "expected silence, got: {lines:#?}");
}

#[test]
fn a_malformed_directive_is_reported_without_disabling_logging() {
    let stderr = run_fixture("fixture_all_levels", "all=info,broken=nonsense");

    assert!(
        stderr.contains("unknown level `nonsense`"),
        "the bad directive should be reported: {stderr}"
    );
    let lines = parse_lines(&stderr);
    assert!(
        !lines.is_empty(),
        "the valid part of the filter must still take effect: {stderr}"
    );
}

#[test]
fn init_is_one_shot() {
    // Installing a global logger must be rejected the second time rather than silently
    // reconfiguring the first.
    init().unwrap();
    let err = init().expect_err("a second init must fail");
    assert!(
        err.to_string().contains("already"),
        "unexpected message: {err}"
    );
}

//}}}
