// (C) 2025 - Enzo Lombardi

//! Validator - input validation system for InputLine controls.
/// Validator module - Input validation for InputLine
/// Matches Borland's TValidator architecture from validate.h and tvalidat.cc
///
/// Validators provide:
/// - Character filtering (which characters are allowed during typing)
/// - Final validation (is the complete value valid?)
/// - Error messages when validation fails
/// - Data transfer (converting between string and typed values)

use std::rc::Rc;
use std::cell::RefCell;

/// Validator options flags
/// Matches Borland's validator option flags (validate.h:21-25)
pub const VO_FILL: u16 = 0x0001;       // Fill with default on empty
pub const VO_TRANSFER: u16 = 0x0002;   // Enable data transfer
pub const VO_ON_APPEND: u16 = 0x0004;  // Validate on each character append

/// Validator status constants
/// Matches Borland's validator status (validate.h:17-20)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorStatus {
    Ok = 0,
    Syntax = 1,
}

/// Base Validator trait
/// Matches Borland's TValidator class (validate.h:36-66)
///
/// All validators implement this trait to provide:
/// - `is_valid()` - Check if the complete input is valid
/// - `is_valid_input()` - Check if input is valid during typing (character by character)
/// - `error()` - Display error message when validation fails
///
/// Reference: local-only/borland-tvision/classes/tvalidat.cc
pub trait Validator {
    /// Check if the complete input string is valid
    /// Matches Borland's TValidator::IsValid() (tvalidat.cc:28-31)
    fn is_valid(&self, input: &str) -> bool;

    /// Check if input is valid during typing
    /// Used to filter characters as user types
    /// Matches Borland's TValidator::IsValidInput() (tvalidat.cc:33-36)
    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        self.is_valid(input)
    }

    /// Display error message when validation fails
    /// Matches Borland's TValidator::Error() - pure virtual in base class
    fn error(&self);

    /// Get validator options
    fn options(&self) -> u16 {
        0
    }

    /// Validate and show error if invalid
    /// Matches Borland's TValidator::Valid() (tvalidat.cc:43-48)
    fn valid(&self, input: &str) -> bool {
        if self.is_valid(input) {
            true
        } else {
            self.error();
            false
        }
    }
}

/// FilterValidator - validates input against a set of allowed characters
/// Matches Borland's TFilterValidator (validate.h:68-92, tfilterv.cc)
///
/// Example:
/// ```
/// use turbo_vision::views::validator::{FilterValidator, Validator};
/// let validator = FilterValidator::new("0123456789"); // Only digits
/// assert!(validator.is_valid("123"));
/// assert!(!validator.is_valid("12a3"));
/// ```
pub struct FilterValidator {
    valid_chars: String,
    options: u16,
}

impl FilterValidator {
    pub fn new(valid_chars: &str) -> Self {
        Self {
            valid_chars: valid_chars.to_string(),
            options: 0,
        }
    }

    pub fn with_options(valid_chars: &str, options: u16) -> Self {
        Self {
            valid_chars: valid_chars.to_string(),
            options,
        }
    }
}

impl Validator for FilterValidator {
    /// Check if all characters in input are in valid_chars
    /// Matches Borland's TFilterValidator::IsValid() (tfilterv.cc:43-52)
    fn is_valid(&self, input: &str) -> bool {
        input.chars().all(|ch| self.valid_chars.contains(ch))
    }

    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        self.is_valid(input)
    }

    fn error(&self) {
        // In a full implementation, this would show a message box
        // For now, just a no-op (the InputLine will handle visual feedback)
        // Matches Borland's TFilterValidator::Error() (tfilterv.cc:59-62)
    }

    fn options(&self) -> u16 {
        self.options
    }
}

/// RangeValidator - validates numeric input within a range
/// Matches Borland's TRangeValidator (validate.h:94-116, trangeva.cc)
///
/// Example:
/// ```
/// use turbo_vision::views::validator::{RangeValidator, Validator};
/// let validator = RangeValidator::new(0, 100); // 0 to 100
/// assert!(validator.is_valid("50"));
/// assert!(!validator.is_valid("150"));
/// assert!(!validator.is_valid("abc"));
/// ```
pub struct RangeValidator {
    min: i64,
    max: i64,
    valid_chars: String,
    options: u16,
}

impl RangeValidator {
    /// Create a new RangeValidator
    /// Matches Borland's TRangeValidator::TRangeValidator(long, long) (trangeva.cc:39-46)
    pub fn new(min: i64, max: i64) -> Self {
        // Determine valid characters based on range
        // Matches Borland's logic in trangeva.cc:28-31,44-45
        let valid_chars = if min >= 0 && max >= 0 {
            // Positive only: no minus sign needed
            "+0123456789xXABCDEFabcdef".to_string()
        } else if min < 0 && max < 0 {
            // Negative only: minus required
            "-0123456789xXABCDEFabcdef".to_string()
        } else {
            // Mixed: both + and - allowed
            "-+0123456789xXABCDEFabcdef".to_string()
        };

        Self {
            min,
            max,
            valid_chars,
            options: 0,
        }
    }

    pub fn with_options(min: i64, max: i64, options: u16) -> Self {
        let mut validator = Self::new(min, max);
        validator.options = options;
        validator
    }

    /// Parse input string to i64, supporting hex (0x) and octal (0) prefixes
    /// Matches Borland's get_val() and get_uval() functions (trangeva.cc:59-69)
    fn parse_value(&self, input: &str) -> Result<i64, std::num::ParseIntError> {
        let trimmed = input.trim();

        // Support hexadecimal (0x prefix)
        if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
            return i64::from_str_radix(&trimmed[2..], 16);
        }

        // Support octal (0 prefix, but not just "0")
        if trimmed.starts_with('0') && trimmed.len() > 1 && !trimmed.contains(|c: char| c == '8' || c == '9') {
            return i64::from_str_radix(&trimmed[1..], 8);
        }

        // Default: decimal
        trimmed.parse::<i64>()
    }
}

impl Validator for RangeValidator {
    /// Check if input is a valid number within range
    /// Matches Borland's TRangeValidator::IsValid() (trangeva.cc:72-91)
    fn is_valid(&self, input: &str) -> bool {
        // First check if characters are valid
        if !input.chars().all(|ch| self.valid_chars.contains(ch)) {
            return false;
        }

        // Try to parse the value
        match self.parse_value(input) {
            Ok(value) => {
                // Check if within range
                value >= self.min && value <= self.max
            }
            Err(_) => false,
        }
    }

    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        // During typing, allow partial input
        // Smart validation: only allow hex letters (a-f, A-F) if user has typed "0x" prefix
        // This prevents confusing UX where users can type letters for decimal ranges like 1-12

        let is_hex_input = input.starts_with("0x") || input.starts_with("0X") ||
                          input.starts_with("+0x") || input.starts_with("+0X") ||
                          input.starts_with("-0x") || input.starts_with("-0X");

        for ch in input.chars() {
            // Always allow: digits, signs, and 'x'/'X' for hex prefix
            if self.valid_chars.contains(ch) &&
               (ch.is_ascii_digit() || ch == '+' || ch == '-' || ch == 'x' || ch == 'X') {
                continue;
            }

            // Only allow hex letters (a-f, A-F) if we're in hex mode
            if ch.is_ascii_alphabetic() && ch != 'x' && ch != 'X' {
                if !is_hex_input {
                    return false;
                }
                // In hex mode, only allow a-f/A-F
                if !matches!(ch.to_ascii_lowercase(), 'a'..='f') {
                    return false;
                }
            } else if !self.valid_chars.contains(ch) {
                return false;
            }
        }

        true
    }

    fn error(&self) {
        // In a full implementation, this would show a message box with range
        // Matches Borland's TRangeValidator::Error() (trangeva.cc:48-57)
        // The message would be: "Value not in the range {min} to {max}"
    }

    fn options(&self) -> u16 {
        self.options
    }
}

/// Type alias for shared validator references
/// InputLine will hold an Option<ValidatorRef>
pub type ValidatorRef = Rc<RefCell<dyn Validator>>;

/// Builder for creating filter validators with a fluent API.
pub struct FilterValidatorBuilder {
    valid_chars: Option<String>,
}

impl FilterValidatorBuilder {
    pub fn new() -> Self {
        Self { valid_chars: None }
    }

    #[must_use]
    pub fn valid_chars(mut self, chars: impl Into<String>) -> Self {
        self.valid_chars = Some(chars.into());
        self
    }

    pub fn build(self) -> FilterValidator {
        let valid_chars = self.valid_chars.expect("FilterValidator valid_chars must be set");
        FilterValidator::new(&valid_chars)
    }

    pub fn build_ref(self) -> ValidatorRef {
        Rc::new(RefCell::new(self.build()))
    }
}

impl Default for FilterValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating range validators with a fluent API.
pub struct RangeValidatorBuilder {
    min: i64,
    max: i64,
}

impl RangeValidatorBuilder {
    pub fn new() -> Self {
        Self { min: 0, max: 100 }
    }

    #[must_use]
    pub fn min(mut self, min: i64) -> Self {
        self.min = min;
        self
    }

    #[must_use]
    pub fn max(mut self, max: i64) -> Self {
        self.max = max;
        self
    }

    #[must_use]
    pub fn range(mut self, min: i64, max: i64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn build(self) -> RangeValidator {
        RangeValidator::new(self.min, self.max)
    }

    pub fn build_ref(self) -> ValidatorRef {
        Rc::new(RefCell::new(self.build()))
    }
}

impl Default for RangeValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_validator_digits() {
        let validator = FilterValidator::new("0123456789");
        assert!(validator.is_valid("123"));
        assert!(validator.is_valid("0"));
        assert!(!validator.is_valid("12a3"));
        assert!(!validator.is_valid("abc"));
    }

    #[test]
    fn test_filter_validator_alphanumeric() {
        let validator = FilterValidator::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        assert!(validator.is_valid("abc123"));
        assert!(validator.is_valid("Test123"));
        assert!(!validator.is_valid("test@example.com"));
    }

    #[test]
    fn test_range_validator_positive() {
        let validator = RangeValidator::new(0, 100);
        assert!(validator.is_valid("0"));
        assert!(validator.is_valid("50"));
        assert!(validator.is_valid("100"));
        assert!(!validator.is_valid("101"));
        assert!(!validator.is_valid("-1"));
        assert!(!validator.is_valid("abc"));
    }

    #[test]
    fn test_range_validator_negative() {
        let validator = RangeValidator::new(-100, -1);
        assert!(validator.is_valid("-1"));
        assert!(validator.is_valid("-50"));
        assert!(validator.is_valid("-100"));
        assert!(!validator.is_valid("0"));
        assert!(!validator.is_valid("-101"));
    }

    #[test]
    fn test_range_validator_mixed() {
        let validator = RangeValidator::new(-50, 50);
        assert!(validator.is_valid("-50"));
        assert!(validator.is_valid("0"));
        assert!(validator.is_valid("50"));
        assert!(!validator.is_valid("-51"));
        assert!(!validator.is_valid("51"));
    }

    #[test]
    fn test_range_validator_hex() {
        let validator = RangeValidator::new(0, 255);
        assert!(validator.is_valid("0xFF"));
        assert!(validator.is_valid("0x00"));
        assert!(validator.is_valid("0xAB"));
        assert!(!validator.is_valid("0x100"));  // 256, out of range
    }

    #[test]
    fn test_range_validator_octal() {
        let validator = RangeValidator::new(0, 100);
        assert!(validator.is_valid("077"));  // 63 in decimal
        assert!(validator.is_valid("0100")); // 64 in decimal
        assert!(!validator.is_valid("0200")); // 128 in decimal, out of range
    }

    #[test]
    fn test_range_validator_no_hex_letters_without_prefix() {
        // Issue #56: For decimal ranges like 1-12, users should not be able
        // to type hex letters (a-f) without first typing "0x"
        let validator = RangeValidator::new(1, 12);

        // Decimal input should work
        assert!(validator.is_valid_input("1", false));
        assert!(validator.is_valid_input("12", false));

        // Hex letters should NOT be allowed without "0x" prefix
        assert!(!validator.is_valid_input("a", false));
        assert!(!validator.is_valid_input("1a", false));
        assert!(!validator.is_valid_input("a1", false));
        assert!(!validator.is_valid_input("f", false));

        // But hex input WITH "0x" prefix should allow letters
        assert!(validator.is_valid_input("0x", false));
        assert!(validator.is_valid_input("0xa", false));
        assert!(validator.is_valid_input("0xA", false));
        assert!(validator.is_valid_input("0xFF", false));
    }

    #[test]
    fn test_range_validator_hex_letters_after_prefix() {
        let validator = RangeValidator::new(0, 255);

        // Hex letters only allowed after "0x" prefix
        assert!(validator.is_valid_input("0x1", false));
        assert!(validator.is_valid_input("0xa", false));
        assert!(validator.is_valid_input("0xAB", false));
        assert!(validator.is_valid_input("0xFF", false));

        // But not without the prefix
        assert!(!validator.is_valid_input("a", false));
        assert!(!validator.is_valid_input("FF", false));

        // Decimal still works
        assert!(validator.is_valid_input("123", false));
        assert!(validator.is_valid_input("255", false));
    }

    #[test]
    fn test_range_validator_hex_with_sign() {
        let validator = RangeValidator::new(-255, 255);

        // Hex with positive sign
        assert!(validator.is_valid_input("+0x", false));
        assert!(validator.is_valid_input("+0xFF", false));

        // Hex with negative sign
        assert!(validator.is_valid_input("-0x", false));
        assert!(validator.is_valid_input("-0xFF", false));

        // Letters not allowed without "0x"
        assert!(!validator.is_valid_input("+a", false));
        assert!(!validator.is_valid_input("-f", false));
    }
}
