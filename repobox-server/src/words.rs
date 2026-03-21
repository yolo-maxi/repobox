use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// List of 100 positive and memorable adjectives
const ADJECTIVES: &[&str] = &[
    "amazing", "awesome", "bright", "brilliant", "calm", "cheerful", "clever", "creative", "curious", "dazzling",
    "delightful", "dynamic", "eager", "elegant", "enchanting", "energetic", "excellent", "exciting", "fabulous", "fantastic",
    "fearless", "friendly", "gentle", "glorious", "graceful", "happy", "harmonious", "healthy", "helpful", "honest",
    "hopeful", "humble", "incredible", "inspiring", "intelligent", "joyful", "keen", "kind", "lovely", "lucky",
    "magical", "marvelous", "mighty", "natural", "noble", "optimistic", "peaceful", "perfect", "playful", "pleasant",
    "positive", "powerful", "precious", "proud", "pure", "quick", "quiet", "radiant", "remarkable", "resilient",
    "serene", "shining", "silent", "simple", "smooth", "soft", "solid", "sparkling", "special", "splendid",
    "steady", "strong", "stunning", "successful", "sunny", "super", "swift", "talented", "tender", "thoughtful",
    "thriving", "tranquil", "tremendous", "triumphant", "trustworthy", "unique", "vibrant", "victorious", "warm", "wise",
    "wonderful", "worthy", "young", "zealous", "bold", "brave", "cool", "fast", "fresh", "great"
];

/// List of 30 colors
const COLORS: &[&str] = &[
    "amber", "azure", "beige", "black", "blue", "brown", "coral", "crimson", "cyan", "emerald",
    "gold", "gray", "green", "indigo", "ivory", "jade", "lavender", "lime", "magenta", "maroon",
    "navy", "olive", "orange", "pink", "purple", "red", "ruby", "silver", "teal", "white",
];

/// List of 100 nouns (animals, nature, objects)
const NOUNS: &[&str] = &[
    "ant", "bear", "bird", "butterfly", "cat", "deer", "dog", "dolphin", "eagle", "elephant",
    "fish", "fox", "frog", "giraffe", "hawk", "horse", "kangaroo", "lion", "monkey", "owl",
    "panda", "rabbit", "shark", "tiger", "turtle", "whale", "wolf", "zebra", "bee", "duck",
    "canyon", "cliff", "cloud", "creek", "field", "forest", "garden", "hill", "island", "lake",
    "meadow", "mountain", "ocean", "peak", "river", "stone", "stream", "tree", "valley", "wave",
    "anchor", "arrow", "bell", "bridge", "castle", "crown", "diamond", "feather", "flame", "flower",
    "gem", "hammer", "key", "lamp", "mirror", "pearl", "prism", "ring", "shield", "star",
    "sword", "tower", "wheel", "wing", "book", "coin", "crystal", "drum", "flute", "globe",
    "harp", "lens", "map", "pen", "scroll", "seed", "shell", "silk", "thread", "vessel",
    "basket", "blade", "chain", "chest", "door", "frame", "glass", "handle", "knot", "orb"
];

/// Generate a random alias using the format: adjective-color-noun
pub fn generate_alias() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use current time as seed for randomness
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let random_value = hasher.finish();

    let adj_idx = (random_value % ADJECTIVES.len() as u64) as usize;
    let color_idx = ((random_value >> 16) % COLORS.len() as u64) as usize;
    let noun_idx = ((random_value >> 32) % NOUNS.len() as u64) as usize;

    format!("{}-{}-{}", ADJECTIVES[adj_idx], COLORS[color_idx], NOUNS[noun_idx])
}

/// Generate a deterministic alias from an address using the format: adjective-color-noun
pub fn generate_alias_from_address(address: &str) -> String {
    let mut hasher = DefaultHasher::new();
    address.hash(&mut hasher);
    let seed = hasher.finish();

    let adj_idx = (seed % ADJECTIVES.len() as u64) as usize;
    let color_idx = ((seed >> 16) % COLORS.len() as u64) as usize;
    let noun_idx = ((seed >> 32) % NOUNS.len() as u64) as usize;

    format!("{}-{}-{}", ADJECTIVES[adj_idx], COLORS[color_idx], NOUNS[noun_idx])
}

/// Check if a string is a valid alias format (adjective-color-noun)
pub fn is_valid_alias(alias: &str) -> bool {
    let parts: Vec<&str> = alias.split('-').collect();

    if parts.len() != 3 {
        return false;
    }

    let adjective_valid = ADJECTIVES.contains(&parts[0]);
    let color_valid = COLORS.contains(&parts[1]);
    let noun_valid = NOUNS.contains(&parts[2]);

    adjective_valid && color_valid && noun_valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_alias() {
        let alias = generate_alias();
        assert!(is_valid_alias(&alias), "Generated alias should be valid: {}", alias);

        // Check format
        let parts: Vec<&str> = alias.split('-').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_generate_alias_from_address() {
        let address = "0x1234567890123456789012345678901234567890";
        let alias1 = generate_alias_from_address(address);
        let alias2 = generate_alias_from_address(address);

        // Should be deterministic
        assert_eq!(alias1, alias2);
        assert!(is_valid_alias(&alias1));
    }

    #[test]
    fn test_is_valid_alias() {
        // Valid aliases
        assert!(is_valid_alias("amazing-blue-cat"));
        assert!(is_valid_alias("clever-green-canyon"));
        assert!(is_valid_alias("amazing-purple-cat")); // Purple IS in colors

        // Invalid aliases
        assert!(!is_valid_alias("invalid-blue-cat")); // Unknown adjective
        assert!(!is_valid_alias("amazing-rainbow-cat")); // Rainbow not in colors
        assert!(!is_valid_alias("amazing-blue")); // Too short
        assert!(!is_valid_alias("amazing-blue-cat-extra")); // Too long
        assert!(!is_valid_alias("amazing_blue_cat")); // Wrong separator
    }

    #[test]
    fn test_different_addresses_generate_different_aliases() {
        let addr1 = "0x1111111111111111111111111111111111111111";
        let addr2 = "0x2222222222222222222222222222222222222222";

        let alias1 = generate_alias_from_address(addr1);
        let alias2 = generate_alias_from_address(addr2);

        // Very likely to be different (not guaranteed due to hash collisions, but extremely likely)
        assert_ne!(alias1, alias2);
    }

    #[test]
    fn test_word_lists_have_correct_sizes() {
        assert_eq!(ADJECTIVES.len(), 100);
        assert_eq!(COLORS.len(), 30);
        assert_eq!(NOUNS.len(), 100);
    }

    #[test]
    fn test_purple_is_in_colors() {
        assert!(COLORS.contains(&"purple"), "Purple should be in colors list");
    }
}