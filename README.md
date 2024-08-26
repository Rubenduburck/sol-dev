# Solana Development Utilities

A collection of utilities and macros to enhance Solana program development, focusing on compute unit logging and instruction discriminant generation.

## Features

- Generate unique instruction discriminants for Anchor programs
- Log compute units for functions and code blocks
- Utility functions for Solana-specific operations

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sol_dev_utils = "0.1.0"
sol_dev_macros = "0.1.0"
sol_dev_proc_macros = "0.1.0"
```

## Usage

### Instruction Discriminants

```rust
use sol_dev_proc_macros::anchor_discriminant;

match discriminant {
    anchor_discriminant![initialize] => initialize(),
    anchor_discriminant![process] => process(),
    anchor_discriminant![custom:finalize] => finalize(),
    _ => return Err(ProgramError::InvalidInstructionData.into()),
}
```

### Compute Unit Logging
- Function-level logging
```rust
use sol_dev_proc_macros::compute_fn;

#[compute_fn]
fn my_expensive_operation() {
    // Function body
}
```
- Code block logging
```rust
use sol_dev_macros::compute_fn;

compute_fn!("My Operation" => {
    // Your code here
    perform_calculation();
});
```

## Contributing
Contributions are welcome! Please open an issue or submit a pull request.

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
