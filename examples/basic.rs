//! A tour of the crate's output.
//!
//! ```console
//! TOPO_LOG=all=trace cargo run --example basic --features trace
//! ```
//!
//! Try narrowing the filter to see prefix matching at work — `all` sets the level used when
//! nothing else matches, and the longest matching target wins:
//!
//! ```console
//! TOPO_LOG=all=warn,basic::solver=trace cargo run --example basic --features trace
//! ```

use topohedral_tracing::{debug, error, info, trace, trace_fn, trace_scope, warn, ENABLED};

mod solver {
    use topohedral_tracing::{debug, info, trace_fn};

    #[trace_fn]
    pub fn solve(input: u32) -> u32 {
        info!("solving for {input}");
        let candidate = refine(input);
        debug!("settled on {candidate}");
        candidate
    }

    #[trace_fn("refine step")]
    fn refine(input: u32) -> u32 {
        for step in 0..2 {
            debug!("refinement step {step}");
        }
        input * 2
    }
}

#[trace_fn]
fn run() {
    trace!("the most verbose level");
    debug!("a debug detail");
    info!("an informational message");
    warn!("something looks off");
    error!("something went wrong");

    {
        trace_scope!("an explicit scope");
        info!("indented one level further");
    }

    let answer = solver::solve(21);
    info!(target: "basic::results", "the answer is {answer}");
}

fn main() {
    topohedral_tracing::init().expect("tracing initialises once");

    if !ENABLED {
        eprintln!(
            "note: built without the `trace` feature, so no trace output will appear.\n      \
             try: TOPO_LOG=all=trace cargo run --example basic --features trace"
        );
    }

    run();
}
