//! Tests that scope records are attributed to the *caller's* source location and target rather
//! than to the tracing crate's own.
//!
//! This is easy to get wrong: `module_path!()` and `file!()` expanded inside the library resolve
//! to the library, so a scope guard that resolves them for itself reports `topohedral_tracing` as
//! the target for every scope in every crate, and no `TOPO_LOG` filter can ever select it.
//!
//! These assertions need the trace points compiled in; the feature-off build is covered by
//! `tests/feature_switch_test.rs`.
#![cfg(feature = "trace")]

mod common;

use common::{line_with, messages, parse_lines, run_fixture};
use topohedral_tracing::{info, init, trace_fn, trace_scope, IndentGuard};

//{{{ fixtures

#[trace_fn]
fn traced_function()
{
    info!("inside traced_function");
    nested_scope();
}

fn nested_scope()
{
    trace_scope!("nested_scope");
    info!("inside nested_scope");
}

fn manual_guard_scope()
{
    // The target is supplied explicitly; `IndentGuard` cannot determine the caller's module.
    let _guard = IndentGuard::new("fixture_target::inner", "manual_guard_scope");
    info!(target: "fixture_target::inner", "inside manual_guard_scope");
}

#[test]
fn fixture_scopes()
{
    if common::skip_fixture()
    {
        return;
    }
    init().unwrap();

    traced_function();
    manual_guard_scope();
}

//}}}
//{{{ tests

#[test]
fn scope_records_name_the_client_file()
{
    let stderr = run_fixture("fixture_scopes", "all=info");
    let lines = parse_lines(&stderr);
    let scopes: Vec<_> = lines
        .iter()
        .filter(|l| l.message.starts_with("* Entering") || l.message.starts_with("* Leaving"))
        .collect();

    assert!(!scopes.is_empty(), "expected scope records:\n{stderr}");
    for line in &scopes
    {
        assert_eq!(
            line.file, "caller_location_test.rs",
            "scope record should point at the client, not the library: {line:?}"
        );
    }
}

#[test]
fn every_scope_is_entered_and_left()
{
    let lines = parse_lines(&run_fixture("fixture_scopes", "all=info"));
    let all = messages(&lines);

    for name in ["traced_function", "nested_scope", "manual_guard_scope"]
    {
        assert!(
            all.contains(&format!("* Entering {name}").as_str()),
            "missing entry for {name}: {all:#?}"
        );
        assert!(
            all.contains(&format!("* Leaving {name}").as_str()),
            "missing exit for {name}: {all:#?}"
        );
    }
}

#[test]
fn trace_fn_and_trace_scope_nest()
{
    let lines = parse_lines(&run_fixture("fixture_scopes", "all=info"));

    assert_eq!(line_with(&lines, "* Entering traced_function").indent, 0);
    assert_eq!(line_with(&lines, "inside traced_function").indent, 1);
    assert_eq!(line_with(&lines, "* Entering nested_scope").indent, 1);
    assert_eq!(line_with(&lines, "inside nested_scope").indent, 2);
}

#[test]
fn a_scope_target_is_the_callers_module_and_is_filterable()
{
    // `trace_scope!` inside this integration test uses `module_path!()` at the call site, which is
    // the test crate's root. Filtering on it must select those records and nothing else.
    let lines = parse_lines(&run_fixture(
        "fixture_scopes",
        "all=off,caller_location_test=info",
    ));
    let all = messages(&lines);

    assert!(
        all.contains(&"* Entering traced_function"),
        "the caller's module must be the scope target: {all:#?}"
    );
    assert!(
        !all.contains(&"* Entering manual_guard_scope"),
        "a scope under a different target must not be selected: {all:#?}"
    );
}

#[test]
fn an_explicit_scope_target_is_matched_by_prefix()
{
    // The guard's target is `fixture_target::inner`; a filter on the parent must match it.
    let lines = parse_lines(&run_fixture(
        "fixture_scopes",
        "all=off,fixture_target=info",
    ));
    let all = messages(&lines);

    assert!(
        all.contains(&"* Entering manual_guard_scope"),
        "a parent-module filter must match a nested target: {all:#?}"
    );
    assert!(
        !all.contains(&"* Entering traced_function"),
        "unrelated targets must stay silent: {all:#?}"
    );
}

//}}}
