#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subject_debug() {
        // Test the exact case from the issue
        let subject = parse_subject("vitalik.eth").unwrap();
        println!("vitalik.eth parsed as: {:?}", subject);
        
        match subject {
            Subject::Identity(id) => println!("✅ Correctly parsed as Identity: {:?}", id),
            Subject::Group(name) => println!("❌ INCORRECTLY parsed as Group: {}", name),
            Subject::All => println!("❌ INCORRECTLY parsed as All"),
        }
        
        // Also test is_ens_name directly
        assert!(is_ens_name("vitalik.eth"), "is_ens_name should return true for vitalik.eth");
        
        // Test other ENS names
        assert!(is_ens_name("alice.eth"));
        assert!(is_ens_name("test.box"));
        assert!(!is_ens_name("notanens"));
        
        // Test parse_subject with other ENS names
        assert!(matches!(parse_subject("alice.eth").unwrap(), Subject::Identity(_)));
        assert!(matches!(parse_subject("test.box").unwrap(), Subject::Identity(_)));
        assert!(matches!(parse_subject("notanens").unwrap(), Subject::Group(_)));
    }
}