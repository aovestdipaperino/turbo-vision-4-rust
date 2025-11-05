// (C) 2025 - Enzo Lombardi
// LookupValidator - Validates input against a list of valid values
//
// Matches Borland: TLookupValidator (validate.h)
//
// Validates that input matches one of a predefined list of valid strings.
// Supports both case-sensitive and case-insensitive matching.

use super::validator::Validator;

/// LookupValidator - Validates against a list of valid values
///
/// Matches Borland: TLookupValidator
pub struct LookupValidator {
    valid_values: Vec<String>,
    case_sensitive: bool,
}

impl LookupValidator {
    /// Create a new lookup validator with a list of valid values
    pub fn new(valid_values: Vec<String>) -> Self {
        Self {
            valid_values,
            case_sensitive: true,
        }
    }

    /// Create a case-insensitive lookup validator
    pub fn new_case_insensitive(valid_values: Vec<String>) -> Self {
        Self {
            valid_values,
            case_sensitive: false,
        }
    }

    /// Set case sensitivity
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    /// Get the list of valid values
    pub fn valid_values(&self) -> &[String] {
        &self.valid_values
    }

    /// Add a valid value
    pub fn add_value(&mut self, value: String) {
        self.valid_values.push(value);
    }

    /// Remove a valid value
    pub fn remove_value(&mut self, value: &str) -> bool {
        if let Some(pos) = self.find_value(value) {
            self.valid_values.remove(pos);
            true
        } else {
            false
        }
    }

    /// Find the position of a value in the list
    fn find_value(&self, value: &str) -> Option<usize> {
        self.valid_values.iter().position(|v| {
            if self.case_sensitive {
                v == value
            } else {
                v.eq_ignore_ascii_case(value)
            }
        })
    }

    /// Check if a value is in the list
    pub fn contains(&self, value: &str) -> bool {
        self.find_value(value).is_some()
    }
}

impl Validator for LookupValidator {
    /// Check if input is in the list of valid values
    /// Matches Borland's TLookupValidator::IsValid()
    fn is_valid(&self, input: &str) -> bool {
        if input.is_empty() {
            // Empty input is allowed - validation happens on non-empty input
            return true;
        }

        self.contains(input)
    }

    /// All characters are allowed - validation is on the complete string
    fn is_valid_input(&self, _input: &str, _append: bool) -> bool {
        true
    }

    /// Display error message when validation fails
    /// Matches Borland's TLookupValidator::Error()
    fn error(&self) {
        // In a full implementation, this would show a message box
        // For now, just a no-op (the InputLine will handle visual feedback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_validator_case_sensitive() {
        let validator = LookupValidator::new(vec![
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
        ]);

        // Valid values
        assert!(validator.is_valid("Red"));
        assert!(validator.is_valid("Green"));
        assert!(validator.is_valid("Blue"));

        // Invalid values
        assert!(!validator.is_valid("red"));
        assert!(!validator.is_valid("RED"));
        assert!(!validator.is_valid("Yellow"));

        // Empty is allowed
        assert!(validator.is_valid(""));
    }

    #[test]
    fn test_lookup_validator_case_insensitive() {
        let validator = LookupValidator::new_case_insensitive(vec![
            "Red".to_string(),
            "Green".to_string(),
            "Blue".to_string(),
        ]);

        // Valid values (any case)
        assert!(validator.is_valid("Red"));
        assert!(validator.is_valid("red"));
        assert!(validator.is_valid("RED"));
        assert!(validator.is_valid("Green"));
        assert!(validator.is_valid("green"));
        assert!(validator.is_valid("BLUE"));

        // Invalid values
        assert!(!validator.is_valid("Yellow"));
        assert!(!validator.is_valid("yellow"));

        // Empty is allowed
        assert!(validator.is_valid(""));
    }

    #[test]
    fn test_lookup_validator_add_remove() {
        let mut validator = LookupValidator::new(vec![
            "Apple".to_string(),
            "Banana".to_string(),
        ]);

        // Initial state
        assert!(validator.is_valid("Apple"));
        assert!(!validator.is_valid("Orange"));

        // Add value
        validator.add_value("Orange".to_string());
        assert!(validator.is_valid("Orange"));

        // Remove value
        assert!(validator.remove_value("Banana"));
        assert!(!validator.is_valid("Banana"));

        // Remove non-existent value
        assert!(!validator.remove_value("NonExistent"));
    }

    #[test]
    fn test_lookup_validator_contains() {
        let validator = LookupValidator::new(vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ]);

        assert!(validator.contains("One"));
        assert!(validator.contains("Two"));
        assert!(validator.contains("Three"));
        assert!(!validator.contains("Four"));
        assert!(!validator.contains("one")); // Case-sensitive
    }

    #[test]
    fn test_lookup_validator_contains_case_insensitive() {
        let validator = LookupValidator::new_case_insensitive(vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
        ]);

        assert!(validator.contains("One"));
        assert!(validator.contains("one"));
        assert!(validator.contains("ONE"));
        assert!(validator.contains("Two"));
        assert!(validator.contains("two"));
        assert!(!validator.contains("Four"));
    }

    #[test]
    fn test_lookup_validator_set_case_sensitive() {
        let mut validator = LookupValidator::new(vec![
            "Test".to_string(),
        ]);

        // Initially case-sensitive
        assert!(validator.is_valid("Test"));
        assert!(!validator.is_valid("test"));

        // Switch to case-insensitive
        validator.set_case_sensitive(false);
        assert!(validator.is_valid("Test"));
        assert!(validator.is_valid("test"));
        assert!(validator.is_valid("TEST"));

        // Switch back to case-sensitive
        validator.set_case_sensitive(true);
        assert!(validator.is_valid("Test"));
        assert!(!validator.is_valid("test"));
    }

    #[test]
    fn test_lookup_validator_valid_values() {
        let validator = LookupValidator::new(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
        ]);

        let values = validator.valid_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "A");
        assert_eq!(values[1], "B");
        assert_eq!(values[2], "C");
    }

    #[test]
    fn test_lookup_validator_is_valid_input() {
        let validator = LookupValidator::new(vec!["Test".to_string()]);

        // All characters are allowed (validation is on complete string)
        assert!(validator.is_valid_input("a", true));
        assert!(validator.is_valid_input("Z", true));
        assert!(validator.is_valid_input("0", true));
        assert!(validator.is_valid_input(" ", true));
        assert!(validator.is_valid_input("!", true));
    }
}
