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

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, ItemFn, LitStr};

/// Instruments a function with automatic trace-scope logging.
///
/// When applied to a function, this attribute inserts a `trace_scope!` call as
/// the first statement in the function body, which logs the function entry at
/// the `Trace` level and indents all subsequent log messages until the function
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
///     let _trace_guard = ::topohedral_tracing::trace_scope!("foo");
///     body
/// }
/// ```
#[proc_macro_attribute]
pub fn trace_fn(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream
{
    let input_fn = parse_macro_input!(item as ItemFn);

    let scope_name = match parse_scope_name(attr, &input_fn)
    {
        Ok(name) => name,
        Err(err) => return err.to_compile_error().into(),
    };

    let attrs = &input_fn.attrs;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let stmts = &input_fn.block.stmts;
    let trace_scope_stmt = quote_spanned! { sig.ident.span()=>
        ::topohedral_tracing::trace_scope!(#scope_name);
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

fn parse_scope_name(
    attr: TokenStream,
    input_fn: &ItemFn,
) -> syn::Result<String>
{
    let attr2: proc_macro2::TokenStream = attr.clone().into();

    if attr2.is_empty()
    {
        return Ok(input_fn.sig.ident.to_string());
    }

    // Try bare string literal: #[trace_fn("custom name")]
    if let Ok(lit) = syn::parse::<LitStr>(attr.clone())
    {
        return Ok(lit.value());
    }

    // Try name = "value": #[trace_fn(name = "custom name")]
    if let Ok(nv) = syn::parse::<syn::MetaNameValue>(attr.clone())
    {
        if nv.path.is_ident("name")
        {
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
        proc_macro2::Span::call_site(),
        "expected #[trace_fn], #[trace_fn(\"name\")], or #[trace_fn(name = \"name\")]",
    ))
}
