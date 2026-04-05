#[cfg(feature = "enable_trace")]
use topohedral_tracing::{info, init, trace_fn, trace_scope, IndentGuard};

#[cfg(feature = "enable_trace")]
const FIXTURE_ENV: &str = "TOPO_CALLER_LOCATION_FIXTURE";

#[cfg(feature = "enable_trace")]
#[trace_fn]
fn traced_function()
{
    info!("inside traced_function");
    nested_scope();
}

#[cfg(feature = "enable_trace")]
fn nested_scope()
{
    trace_scope!("nested_scope");
    info!("inside nested_scope");
}

#[cfg(feature = "enable_trace")]
fn manual_guard_scope()
{
    let _guard = IndentGuard::new("manual_guard_scope".to_string());
    info!("inside manual_guard_scope");
}

#[cfg(feature = "enable_trace")]
#[test]
fn caller_location_fixture()
{
    if std::env::var_os(FIXTURE_ENV).is_none()
    {
        return;
    }

    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    traced_function();
    manual_guard_scope();
}

#[cfg(feature = "enable_trace")]
#[test]
fn test_trace_scope_and_trace_fn_use_client_locations()
{
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("caller_location_fixture")
        .arg("--nocapture")
        .env(FIXTURE_ENV, "1")
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "fixture failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let scope_lines: Vec<&str> = stderr
        .lines()
        .filter(|line| line.contains("* Entering") || line.contains("* Leaving"))
        .collect();

    assert!(
        !scope_lines.is_empty(),
        "expected scope logging in stderr:\n{stderr}"
    );

    for line in &scope_lines
    {
        assert!(
            line.contains("caller_location_test:"),
            "expected client file location in line: {line}"
        );
        assert!(
            !line.contains(" lib:"),
            "expected not to log tracing crate file in line: {line}"
        );
    }

    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Entering traced_function")),
        "missing traced function entry line:\n{stderr}"
    );
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Leaving traced_function")),
        "missing traced function exit line:\n{stderr}"
    );
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Entering nested_scope")),
        "missing nested scope entry line:\n{stderr}"
    );
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Leaving nested_scope")),
        "missing nested scope exit line:\n{stderr}"
    );
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Entering manual_guard_scope")),
        "missing manual guard entry line:\n{stderr}"
    );
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("* Leaving manual_guard_scope")),
        "missing manual guard exit line:\n{stderr}"
    );
}
