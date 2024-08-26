use sha2::Digest;

/// The discriminant for a function is sha-256(function_name)[..8]
pub fn anchor_discriminant(namespace: &str, function_name: &str) -> [u8; 8] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{}:{}", namespace, function_name));
    let result = hasher.finalize();
    result[..8].try_into().unwrap()
}

