#[cfg(test)]
mod tests {
    #[test]
    fn test_discriminant_fn() {
        assert_eq!(
            sol_dev_utils::anchor_discriminant("global", "initialize"),
            [175, 175, 109, 31, 13, 152, 155, 237]
        );
    }
    #[test]
    fn test_discriminant_macro() {
        assert_eq!(
            sol_dev_proc_macros::discriminant![initialize],
            [175, 175, 109, 31, 13, 152, 155, 237]
        );
    }
}

