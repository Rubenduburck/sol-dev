extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Generates a discriminant for a given function name within a global namespace.
///
/// This macro is used to create a unique identifier (discriminant) for a function
/// within the Anchor framework. It's typically used for generating unique
/// instruction identifiers in Solana programs.
///
/// Anchor typically requires adds the namespace to the function name to generate
/// a unique identifier. In this macro, we default to the global namespace if no
/// namespace is provided.
///
/// # Arguments
///
/// * `input` - A `TokenStream` containing the function name to generate a discriminant for.
///
/// # Returns
///
/// A `TokenStream` representing an array of bytes, which is the generated discriminant.
///
/// # Example
///
/// ```rust
/// use sol_dev_proc_macros::anchor_discriminant;
/// const DISCRIMINANT: [u8; 8] = anchor_discriminant!(initialize);
/// const DISCRIMINANT_WITH_NAMESPACE: [u8; 8] = anchor_discriminant!(global:initialize);
/// assert_eq!(
///    DISCRIMINANT,
///    [175, 175, 109, 31, 13, 152, 155, 237]
/// );
/// assert_eq!(
///     DISCRIMINANT_WITH_NAMESPACE,
///     DISCRIMINANT
/// );
/// ```
#[proc_macro]
pub fn anchor_discriminant(input: TokenStream) -> TokenStream {
    const NAMESPACE: &str = "global";
    let function_name = input.to_string();
    // If the function does not contain a namespace, we add the global namespace.
    let full_name = if function_name.contains(':') {
        function_name
    } else {
        format!("{}:{}", NAMESPACE, function_name)
    };
    let arr = sol_dev_utils::anchor_discriminant(&full_name);
    let expanded = quote::quote! {
        [#(#arr),*]
    };
    TokenStream::from(expanded)
}

/// Attribute macro for instrumenting functions with compute unit logging.
///
/// This macro wraps the decorated function with additional logging statements
/// that print the function name and the number of compute units used before and after
/// the function execution.
///
/// # Usage
///
/// ```rust,ignore
/// #[compute_fn]
/// fn my_function() {
///     // Function body
/// }
/// ```
///
/// # Effects
///
/// - Adds a log message with the function name at the start of execution.
/// - Logs the number of compute units before and after the function execution.
/// - Adds a closing log message with the function name at the end of execution.
///
/// # Note
///
/// Total extra compute units used per `compute_fn!` call: 409 CU
/// For more details, see:
/// - https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/programs/bpf_loader/src/syscalls/logging.rs#L70
/// - https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/program-runtime/src/compute_budget.rs#L150
#[proc_macro_attribute]
pub fn compute_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let block = &input.block;

    input.block = syn::parse_quote!({
        ::solana_program::msg!(concat!(stringify!(#fn_name), " {"));
        ::solana_program::log::sol_log_compute_units();

        let __result = (|| #block)();

        ::solana_program::log::sol_log_compute_units();
        ::solana_program::msg!(concat!("} // ", stringify!(#fn_name)));

        __result
    });

    quote!(#input).into()
}
