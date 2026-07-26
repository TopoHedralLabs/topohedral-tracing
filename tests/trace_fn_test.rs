//! End-to-end tests for the `#[trace_fn]` attribute macro.
//!
//! These assertions need the trace points compiled in; the feature-off build is covered by
//! `tests/feature_switch_test.rs`.
#![cfg(feature = "trace")]

mod common;

use common::{line_with, messages, parse_lines, run_fixture};
use topohedral_tracing::{info, init, trace_fn};

//{{{ traced functions

#[trace_fn]
fn simple_function() -> i32 {
    info!("body of simple_function");
    42
}

#[trace_fn("custom_name")]
fn function_with_custom_name() -> &'static str {
    "hello"
}

#[trace_fn(name = "named_custom")]
fn function_with_name_eq() {}

#[trace_fn]
fn function_with_args(
    x: i32,
    y: i32,
) -> i32 {
    x + y
}

#[trace_fn]
fn generic_function<T: std::fmt::Debug>(value: T) -> String {
    format!("{value:?}")
}

#[trace_fn]
fn nested_inner() {}

#[trace_fn]
fn nested_outer() {
    nested_inner();
}

#[trace_fn]
fn returns_early(flag: bool) -> i32 {
    if flag {
        return 1;
    }
    2
}

//}}}
//{{{ fixtures

#[test]
fn fixture_traced_functions() {
    if common::skip_fixture() {
        return;
    }
    init().unwrap();

    assert_eq!(simple_function(), 42);
    assert_eq!(function_with_custom_name(), "hello");
    function_with_name_eq();
    assert_eq!(function_with_args(3, 4), 7);
    assert_eq!(generic_function(7u8), "7");
    assert_eq!(returns_early(true), 1);
    nested_outer();
}

//}}}
//{{{ tests

#[test]
fn the_function_name_is_the_default_scope_name() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));

    assert_eq!(line_with(&lines, "* Entering simple_function").indent, 0);
    assert_eq!(line_with(&lines, "body of simple_function").indent, 1);
    assert_eq!(line_with(&lines, "* Leaving simple_function").indent, 0);
}

#[test]
fn both_custom_name_syntaxes_are_honoured() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));
    let all = messages(&lines);

    assert!(all.contains(&"* Entering custom_name"), "{all:#?}");
    assert!(all.contains(&"* Leaving custom_name"), "{all:#?}");
    assert!(all.contains(&"* Entering named_custom"), "{all:#?}");
    assert!(
        !all.iter().any(|m| m.contains("function_with_custom_name")),
        "the custom name should replace the function name: {all:#?}"
    );
}

#[test]
fn functions_with_arguments_and_generics_are_instrumented() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));
    let all = messages(&lines);

    assert!(all.contains(&"* Entering function_with_args"), "{all:#?}");
    assert!(all.contains(&"* Entering generic_function"), "{all:#?}");
}

#[test]
fn an_early_return_still_logs_the_exit() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));
    let all = messages(&lines);

    let enter = all.iter().position(|m| *m == "* Entering returns_early");
    let leave = all.iter().position(|m| *m == "* Leaving returns_early");
    assert!(
        matches!((enter, leave), (Some(e), Some(l)) if l == e + 1),
        "entry should be followed immediately by exit: {all:#?}"
    );
}

#[test]
fn nested_traced_functions_indent() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));

    assert_eq!(line_with(&lines, "* Entering nested_outer").indent, 0);
    assert_eq!(line_with(&lines, "* Entering nested_inner").indent, 1);
    assert_eq!(line_with(&lines, "* Leaving nested_inner").indent, 1);
    assert_eq!(line_with(&lines, "* Leaving nested_outer").indent, 0);
}

#[test]
fn records_are_attributed_to_the_traced_function_not_the_macro() {
    let lines = parse_lines(&run_fixture("fixture_traced_functions", "all=info"));

    assert!(
        lines.iter().all(|l| l.file == "trace_fn_test.rs"),
        "every record should point at this file: {lines:#?}"
    );
    assert!(
        line_with(&lines, "* Entering simple_function").line > 0,
        "a line number should be recorded"
    );
}

//}}}
