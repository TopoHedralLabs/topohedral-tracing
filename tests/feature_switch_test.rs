//! Tests that hold in **both** configurations of the compile-time switch.
//!
//! This file is deliberately not gated on the `trace` feature. Its job is to prove that every
//! macro compiles and behaves consistently whether or not trace points are compiled in — the
//! feature-off build being the default that every consumer gets, and the one that previously went
//! entirely untested.

mod common;

use common::{parse_lines, run_fixture};
use topohedral_tracing::{
    debug, error, increment_indent, indent_level, info, init, trace, trace_fn, trace_scope,
    IndentGuard, ENABLED,
};

//{{{ compile-time surface

#[trace_fn]
fn traced() -> i32
{
    trace!("inside traced");
    7
}

#[trace_fn("renamed")]
fn traced_with_name() {}

fn scoped() -> i32
{
    trace_scope!("scoped");
    computed_name(3);
    traced()
}

fn computed_name(block: u32)
{
    // A literal name borrows; a computed one allocates. Both must be accepted.
    trace_scope!(format!("block {block}"));
}

//}}}
//{{{ fixtures

#[test]
fn fixture_emit_everything()
{
    if common::skip_fixture()
    {
        return;
    }
    init().unwrap();

    assert_eq!(scoped(), 7);
    traced_with_name();
    info!("a message");
    error!(target: "custom", "a targeted message");

    // Not one of this crate's macros: a dependency logging through the facade.
    log::warn!("via the log facade");
}

//}}}
//{{{ tests

#[test]
fn enabled_reflects_this_crates_feature_not_the_callers()
{
    // This test crate declares no feature of its own, yet must agree with the library. Under the
    // old `#[cfg(feature = ..)]`-in-macro-body design the switch was resolved against the calling
    // crate, so this is the property that regressed.
    assert_eq!(ENABLED, cfg!(feature = "trace"));
}

#[test]
fn every_macro_compiles_and_evaluates_to_unit()
{
    // Statement, expression and tail positions must all type-check identically in both
    // configurations.
    let unit: () = info!("unit valued");
    assert_eq!(unit, ());

    match 1
    {
        0 => trace!("in the zero arm"),
        _ => trace!("in the fallback arm"),
    }

    if true
    {
        debug!("in a branch");
    }

    let _closure = || error!(target: "custom", "in a closure");

    #[allow(clippy::let_unit_value)]
    let _targeted: () = debug!(target: "custom", "targeted");
}

#[test]
fn indentation_helpers_are_always_callable()
{
    // These are ordinary functions rather than macros, so they exist in both configurations.
    let before = indent_level();
    increment_indent();
    assert_eq!(indent_level(), before + 1);
    topohedral_tracing::decrement_indent();
    assert_eq!(indent_level(), before);
}

#[test]
fn a_guard_can_be_constructed_in_either_configuration()
{
    let guard = IndentGuard::new("feature_switch_test", "explicit");
    // The guard raises indentation only when trace points are compiled in; either way the type
    // exists and has a single constructor signature.
    assert_eq!(indent_level(), usize::from(ENABLED));
    drop(guard);
    assert_eq!(indent_level(), 0);
}

#[test]
fn this_crates_trace_points_follow_the_switch()
{
    let lines = parse_lines(&run_fixture("fixture_emit_everything", "all=trace"));
    let ours: Vec<&str> = lines
        .iter()
        .map(|l| l.message.as_str())
        .filter(|m| *m != "via the log facade")
        .collect();

    if ENABLED
    {
        assert!(!ours.is_empty(), "expected output when the switch is on");
        assert!(ours.contains(&"* Entering scoped"), "{ours:#?}");
        assert!(
            ours.contains(&"* Entering renamed"),
            "custom scope names should be honoured: {ours:#?}"
        );
        assert!(
            ours.contains(&"* Entering block 3"),
            "computed scope names should be honoured: {ours:#?}"
        );
    }
    else
    {
        assert!(
            ours.is_empty(),
            "the switch is off, so this crate's trace points should emit nothing: {ours:#?}"
        );
    }
}

#[test]
fn the_log_facade_is_served_in_both_configurations()
{
    // The compile-time switch erases *this crate's* macros. It cannot erase a dependency's
    // `log::` calls, and it should not try to: the installed backend stays useful in non-tracing
    // builds, which is where a dependency's warnings and errors matter most.
    let lines = parse_lines(&run_fixture("fixture_emit_everything", "all=trace"));

    assert!(
        lines.iter().any(|l| l.message == "via the log facade"),
        "facade records must survive with ENABLED={ENABLED}: {lines:#?}"
    );
}

//}}}
