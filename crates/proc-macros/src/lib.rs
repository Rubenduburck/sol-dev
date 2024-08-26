extern crate proc_macro;
use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro]
pub fn discriminant(input: TokenStream) -> TokenStream {
    const NAMESPACE: &str = "global";
    let function_name = input.to_string();
    let arr = sol_dev_utils::anchor_discriminant(NAMESPACE, &function_name);
    let expanded = quote::quote! {
        [#(#arr),*]
    };
    TokenStream::from(expanded)
}

/// Total extra compute units used per compute_fn! call 409 CU
/// https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/programs/bpf_loader/src/syscalls/logging.rs#L70
/// https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/program-runtime/src/compute_budget.rs#L150
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
