use sha2::Digest;

/// Calculates the discriminant for a function using SHA-256 hash.
///
/// The discriminant is defined as the first 8 bytes of the SHA-256 hash of the function name.
///
/// # Arguments
///
/// * `input` - A string slice that holds the function name.
///
/// # Returns
///
/// An array of 8 bytes representing the discriminant.
///
/// # Examples
///
/// ```
/// let discriminant = sol_dev_utils::anchor_discriminant("initialize");
/// assert_eq!(discriminant, [175, 175, 109, 31, 13, 152, 155, 237]);
/// ```
pub fn anchor_discriminant(input: &str) -> [u8; 8] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    let result = hasher.finalize();
    result[..8].try_into().unwrap()
}

