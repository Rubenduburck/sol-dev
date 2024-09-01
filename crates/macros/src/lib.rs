/// Original [here](https://github.com/thlorenz/sol-contracts/blob/master/packages/sol-common/rust/src/lib.rs)
/// A macro for logging compute units used by a specific code block.
///
/// This macro wraps a code block, logging the compute units before and after its execution.
/// It also adds opening and closing log messages for easier tracking in the program output.
///
/// # Arguments
///
/// * `$msg` - A string literal used as a label for the logged block.
/// * `$($tt)*` - The code block to be executed and measured.
///
/// # Returns
///
/// Returns the result of the executed code block.
///
/// # Note
///
/// This macro consumes an additional 409 compute units per call due to the logging operations.
///
/// # Examples
///
/// ```rust,ignore
/// use solana_program;
/// sol_dev_macros::compute_fn!("My Operation" => {
///     // Your code here
///     42
/// });
/// ```
///
/// # References
///
/// * [Logging syscall](https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/programs/bpf_loader/src/syscalls/logging.rs#L70)
/// * [Compute budget](https://github.com/anza-xyz/agave/blob/d88050cda335f87e872eddbdf8506bc063f039d3/program-runtime/src/compute_budget.rs#L150)
///
#[macro_export]
macro_rules! compute_fn {
    ($msg:expr=> $($tt:tt)*) => {
        ::solana_program::msg!(concat!($msg, " {"));
        ::solana_program::log::sol_log_compute_units();
        let res = { $($tt)* };
        ::solana_program::log::sol_log_compute_units();
        ::solana_program::msg!(concat!(" } // ", $msg));
        res
    };
}
