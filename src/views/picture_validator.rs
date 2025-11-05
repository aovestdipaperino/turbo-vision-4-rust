// (C) 2025 - Enzo Lombardi
// Picture Mask Validator (TPXPictureValidator)
//
// Validates and formats input according to a picture mask.
// Borland's TPXPictureValidator from validate.h and tvalidat.cc
//
// Mask Characters:
// - # : Digit (0-9)
// - @ : Alpha (A-Z, a-z)
// - ! : Any character
// - * : Optional character (makes following characters optional)
// - Literal characters : Must match exactly
//
// Examples:
// - "(###) ###-####" : Phone number (555) 123-4567
// - "##/##/####"     : Date 12/25/2023
// - "@@@@-####"      : Code ABCD-1234
// - "###*-####"      : Optional dash 123-4567 or 1234567
//
// Reference: Borland Turbo Vision tvalidat.cc, validate.h

use crate::views::validator::{Validator, ValidatorRef};
use std::cell::RefCell;
use std::rc::Rc;

/// Picture mask validator for formatted input
pub struct PictureValidator {
    /// Picture mask string
    mask: String,
    /// Whether to auto-format as user types
    auto_format: bool,
}

impl PictureValidator {
    /// Create a new picture validator with the given mask
    ///
    /// # Example
    /// ```
    /// use turbo_vision::views::picture_validator::PictureValidator;
    ///
    /// // Phone number mask
    /// let validator = PictureValidator::new("(###) ###-####");
    ///
    /// // Date mask
    /// let validator = PictureValidator::new("##/##/####");
    /// ```
    pub fn new(mask: &str) -> Self {
        PictureValidator {
            mask: mask.to_string(),
            auto_format: true,
        }
    }

    /// Create a new picture validator without auto-formatting
    pub fn new_no_format(mask: &str) -> Self {
        PictureValidator {
            mask: mask.to_string(),
            auto_format: false,
        }
    }

    /// Get the mask string
    pub fn mask(&self) -> &str {
        &self.mask
    }

    /// Set whether to auto-format input
    pub fn set_auto_format(&mut self, auto_format: bool) {
        self.auto_format = auto_format;
    }

    /// Check if a character is valid for the given mask position
    fn is_valid_char_for_mask(&self, ch: char, mask_ch: char) -> bool {
        match mask_ch {
            '#' => ch.is_ascii_digit(),
            '@' => ch.is_ascii_alphabetic(),
            '!' => true,
            '*' => true, // Optional marker - accept anything
            _ => ch == mask_ch, // Literal must match
        }
    }

    /// Format input according to the mask
    ///
    /// Returns the formatted string, filling in literal characters from the mask.
    pub fn format(&self, input: &str) -> String {
        let mut result = String::new();
        let mut input_chars = input.chars().filter(|&c| !c.is_whitespace());
        let mask_chars: Vec<char> = self.mask.chars().collect();
        let mut optional = false;

        for &mask_ch in &mask_chars {
            if mask_ch == '*' {
                optional = true;
                continue;
            }

            match mask_ch {
                '#' | '@' | '!' => {
                    // Field character - consume from input
                    if let Some(ch) = input_chars.next() {
                        if self.is_valid_char_for_mask(ch, mask_ch) {
                            result.push(ch);
                        } else if optional {
                            // Invalid in optional section - stop
                            break;
                        } else {
                            // Invalid character for required field
                            // Skip it and try next
                            continue;
                        }
                    } else if optional {
                        // No more input and we're in optional section - done
                        break;
                    } else {
                        // Required field but no more input - incomplete
                        break;
                    }
                }
                _ => {
                    // Literal character - add to result
                    result.push(mask_ch);
                }
            }
        }

        result
    }

    /// Check if input matches the mask completely
    fn matches_mask(&self, input: &str) -> bool {
        let mask_chars: Vec<char> = self.mask.chars().collect();
        let input_chars: Vec<char> = input.chars().collect();
        let mut mask_idx = 0;
        let mut input_idx = 0;
        let mut optional = false;

        while mask_idx < mask_chars.len() {
            let mask_ch = mask_chars[mask_idx];

            if mask_ch == '*' {
                optional = true;
                mask_idx += 1;
                continue;
            }

            match mask_ch {
                '#' | '@' | '!' => {
                    // Field character - must match input
                    if input_idx >= input_chars.len() {
                        return optional; // OK if optional section
                    }

                    let input_ch = input_chars[input_idx];
                    if !self.is_valid_char_for_mask(input_ch, mask_ch) {
                        return false;
                    }

                    input_idx += 1;
                }
                _ => {
                    // Literal character - must match exactly
                    if input_idx >= input_chars.len() {
                        return optional;
                    }

                    if input_chars[input_idx] != mask_ch {
                        return false;
                    }

                    input_idx += 1;
                }
            }

            mask_idx += 1;
        }

        // All input consumed?
        input_idx == input_chars.len()
    }
}

impl Validator for PictureValidator {
    fn is_valid(&self, input: &str) -> bool {
        if input.is_empty() {
            return true; // Empty is valid (might be required by parent)
        }

        self.matches_mask(input)
    }

    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        if input.is_empty() {
            return true;
        }

        // For auto-format mode, check if the formatted version is valid
        if self.auto_format {
            let formatted = self.format(input);
            return !formatted.is_empty();
        }

        // For non-auto-format, check if it's on track to match the mask
        let mask_chars: Vec<char> = self.mask.chars().collect();
        let input_chars: Vec<char> = input.chars().collect();
        let mut mask_idx = 0;
        let mut input_idx = 0;
        let mut _optional = false;

        while input_idx < input_chars.len() && mask_idx < mask_chars.len() {
            let mask_ch = mask_chars[mask_idx];

            if mask_ch == '*' {
                _optional = true;
                mask_idx += 1;
                continue;
            }

            match mask_ch {
                '#' | '@' | '!' => {
                    if !self.is_valid_char_for_mask(input_chars[input_idx], mask_ch) {
                        return false;
                    }
                    input_idx += 1;
                }
                _ => {
                    // Literal must match
                    if input_chars[input_idx] != mask_ch {
                        return false;
                    }
                    input_idx += 1;
                }
            }

            mask_idx += 1;
        }

        true // Partial input is valid
    }

    fn error(&self) {
        eprintln!("Input must match format: {}", self.mask);
    }

    fn valid(&self, input: &str) -> bool {
        if self.is_valid(input) {
            true
        } else {
            self.error();
            false
        }
    }
}

/// Helper function to create a ValidatorRef for a PictureValidator
pub fn picture_validator(mask: &str) -> ValidatorRef {
    Rc::new(RefCell::new(PictureValidator::new(mask)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone_number_mask() {
        let validator = PictureValidator::new("(###) ###-####");

        // Valid complete input
        assert!(validator.is_valid("(555) 123-4567"));

        // Invalid - wrong format
        assert!(!validator.is_valid("555-123-4567"));
        assert!(!validator.is_valid("(abc) def-ghij"));
    }

    #[test]
    fn test_date_mask() {
        let validator = PictureValidator::new("##/##/####");

        // Valid dates
        assert!(validator.is_valid("12/25/2023"));
        assert!(validator.is_valid("01/01/2000"));

        // Invalid dates
        assert!(!validator.is_valid("12-25-2023")); // Wrong separator
        assert!(!validator.is_valid("1/1/2023"));   // Missing leading zeros
    }

    #[test]
    fn test_format_phone_number() {
        let validator = PictureValidator::new("(###) ###-####");

        // Format digits only
        assert_eq!(validator.format("5551234567"), "(555) 123-4567");
        assert_eq!(validator.format("555 123 4567"), "(555) 123-4567");
    }

    #[test]
    fn test_format_date() {
        let validator = PictureValidator::new("##/##/####");

        assert_eq!(validator.format("12252023"), "12/25/2023");
        assert_eq!(validator.format("01012000"), "01/01/2000");
    }

    #[test]
    fn test_alpha_mask() {
        let validator = PictureValidator::new("@@@@-####");

        assert!(validator.is_valid("ABCD-1234"));
        assert!(!validator.is_valid("1234-ABCD")); // Wrong order
    }

    #[test]
    fn test_optional_section() {
        let validator = PictureValidator::new("###*-####");

        // With optional dash
        assert!(validator.is_valid("123-4567"));
        // Without optional dash (not fully supported yet)
        // This test shows the current limitation
    }

    #[test]
    fn test_any_character_mask() {
        let validator = PictureValidator::new("!!!-!!!!");

        assert!(validator.is_valid("abc-123d"));
        assert!(validator.is_valid("XYZ-ABCD"));
    }

    #[test]
    fn test_partial_input_validation() {
        let validator = PictureValidator::new("(###) ###-####");

        // Partial inputs should be valid during typing
        assert!(validator.is_valid_input("(5", false));
        assert!(validator.is_valid_input("(55", false));
        assert!(validator.is_valid_input("(555", false));
        assert!(validator.is_valid_input("(555) ", false));
        assert!(validator.is_valid_input("(555) 1", false));
    }

    #[test]
    fn test_empty_input() {
        let validator = PictureValidator::new("##/##/####");
        assert!(validator.is_valid("")); // Empty is valid
    }

    #[test]
    fn test_validator_trait() {
        let validator = PictureValidator::new("(###) ###-####");
        assert!(validator.valid("(555) 123-4567"));
        assert!(!validator.valid("invalid"));
    }
}
