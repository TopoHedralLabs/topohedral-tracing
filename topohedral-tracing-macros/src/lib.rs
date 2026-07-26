//! Procedural macros for topohedral-tracing.
//!
//! This crate provides the `#[trace_fn]` attribute macro which automatically
//! instruments functions with `trace_scope!` for call-stack tracing.
//!
//! This crate should not be used directly. Instead, use the re-export from
//! `topohedral-tracing`:
//!
//! ```ignore
//! use topohedral_tracing::trace_fn;
//!
//! #[trace_fn]
//! fn my_function() {
//!     // automatically traced
//! }
//! ```
//!
//! (The example above cannot be a real doctest: a proc-macro crate's own doctests cannot depend on
//! the crate that re-exports it. Tested examples live in `topohedral-tracing` itself.)

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Ident, ItemFn, LitStr};

/// Instruments a function with automatic scope logging.
///
/// When applied to a function, this attribute inserts a `trace_scope!` call as
/// the first statement in the function body, which logs the function entry at
/// the `Info` level and indents all subsequent log messages until the function
/// returns.
///
/// # Usage
///
/// ```ignore
/// use topohedral_tracing::trace_fn;
///
/// #[trace_fn]                     // uses function name "my_function"
/// fn my_function() { /* ... */ }
///
/// #[trace_fn("custom name")]     // uses custom name "custom name"
/// fn other_function() { /* ... */ }
///
/// #[trace_fn(name = "custom")]   // alternative syntax for custom name
/// fn another_function() { /* ... */ }
/// ```
///
/// # Expansion
///
/// `#[trace_fn]` on `fn foo() { body }` expands to:
///
/// ```ignore
/// fn foo() {
///     ::topohedral_tracing::trace_scope!("foo");
///     body
/// }
/// ```
///
/// `trace_scope!` is a statement macro that declares its own guard binding; the guard lives until
/// the end of the function body, so exit is logged on every path out, including `?` and unwinding.
///
/// # `async fn`
///
/// The guard is held across `.await` points. Entry and exit therefore bracket the whole future
/// rather than each poll, and concurrent tasks on the same thread interleave their indentation.
/// `IndentGuard` is `!Send`, so a traced `async fn` cannot be spawned onto a work-stealing
/// executor that requires `Send` futures.
#[proc_macro_attribute]
pub fn trace_fn(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let scope_name = match parse_scope_name(attr, &input_fn) {
        Ok(name) => name,
        Err(err) => return err.to_compile_error().into(),
    };

    let krate = tracing_crate_path();
    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let stmts = &input_fn.block.stmts;
    // Span the inserted statement at the function name so that the `file!()`/`line!()` captured
    // by `trace_scope!` point at the function being traced.
    let trace_scope_stmt = quote_spanned! { sig.ident.span()=>
        #krate::trace_scope!(#scope_name);
    };

    let output = quote! {
        #(#attrs)*
        #vis #sig {
            #trace_scope_stmt
            #(#stmts)*
        }
    };

    output.into()
}

/// Resolves the path by which the calling crate can name `topohedral-tracing`.
///
/// Hard-coding `::topohedral_tracing` breaks for consumers that rename the dependency
/// (`tracing = { package = "topohedral-tracing" }`), so the real name is read from their
/// `Cargo.toml`.
fn tracing_crate_path() -> TokenStream2 {
    match crate_name("topohedral-tracing") {
        Ok(FoundCrate::Itself) => {
            // `Itself` is also reported for the crate's own integration tests, examples and
            // benches. Those are separate crates that link `topohedral-tracing` externally, so
            // `crate::` would not resolve there; only the lib target itself can use it.
            if std::env::var("CARGO_CRATE_NAME").as_deref() == Ok("topohedral_tracing") {
                quote!(crate)
            } else {
                quote!(::topohedral_tracing)
            }
        }
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        // No manifest to consult (rustdoc, some build setups); the canonical name is the best
        // guess available.
        Err(_) => quote!(::topohedral_tracing),
    }
}

fn parse_scope_name(
    attr: TokenStream,
    input_fn: &ItemFn,
) -> syn::Result<String> {
    let attr2: TokenStream2 = attr.clone().into();

    if attr2.is_empty() {
        return Ok(input_fn.sig.ident.to_string());
    }

    // Try bare string literal: #[trace_fn("custom name")]
    if let Ok(lit) = syn::parse::<LitStr>(attr.clone()) {
        return Ok(lit.value());
    }

    // Try name = "value": #[trace_fn(name = "custom name")]
    if let Ok(nv) = syn::parse::<syn::MetaNameValue>(attr.clone()) {
        if nv.path.is_ident("name") {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(ref s),
                ..
            }) = nv.value
            {
                return Ok(s.value());
            }
        }
        return Err(syn::Error::new_spanned(
            nv.path,
            "expected `name = \"...\"`",
        ));
    }

    Err(syn::Error::new(
        Span::call_site(),
        "expected #[trace_fn], #[trace_fn(\"name\")], or #[trace_fn(name = \"name\")]",
    ))
}
