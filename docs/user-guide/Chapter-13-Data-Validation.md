# Chapter 13: Data Validation

Data validation in Turbo Vision ensures that user input meets specific criteria before being accepted by your application. The validation system is flexible and extensible, supporting multiple validation strategies.

It's important to remember that the validation is handled by validator objects, not by the input line objects themselves. If you've already created a customized input line for a specialized purpose, you've probably already duplicated capability that's built into input lines and their validators.

The "How validators work" section of this chapter describes the various ways in which input line objects automatically call on validator objects.

## Validation Strategies

Turbo Vision provides several complementary validation strategies:

### Filtering Input

The simplest way to ensure that a field contains valid data is to ensure that the user can only type valid data. Turbo Vision provides filter validators that enable you to restrict the characters the user can type. For example, a numeric input field might restrict the user to typing only numeric digits.

Filter validator objects provide a generic mechanism for limiting the characters a user can type in a given input line. Picture validator objects can also control the formatting and types of characters a user can type.

### Validating Each Field

Sometimes you'll find it necessary to ensure that the user types valid input in a particular field before moving to the next field. This approach is often called "validate on Tab," since pressing Tab is the usual way to move the input focus in a data entry screen.

An example is an application that performs a lookup from a database, where the user types in some kind of key information in a field, and the application responds by retrieving the appropriate record and filling the rest of the fields. In such a case, your application needs to check that the user has typed the proper information in the key field before acting on that key.

The input line's validator can control individual field validation. When a view loses input focus and validation fails, the validator alerts the user and keeps the focus in the field until the user provides valid data.

### Validating Full Screens

You can handle validation of full data screens in three different ways:

#### Validating Modal Windows

When a user closes a modal window (such as a dialog), the window can automatically validate all its subviews before closing, unless the closing command was Cancel. To validate its subviews, the window calls each subview's validation method, and if each returns true, the window can close. If any of the subviews returns false, the window is not allowed to close.

A modal window with invalid data can only be canceled until the user provides valid data.

#### Validating on Focus Change

As with any view, you can configure validation to occur when a window loses focus. If you use a modeless data entry window, you can force it to validate its subviews when the window loses focus, such as when you select another window with the mouse. This prevents you from moving to another window that might act on the data entry window's data before you've validated those data.

#### Validating on Demand

You can tell a window to validate all its subviews at any time by calling appropriate validation methods. This essentially asks the window "If I told you to close right now, would all your fields be valid?" The window checks the validation of all its subviews and returns true if all of them are valid.

Calling validation does not obligate you to actually close the window. For example, you might validate when the user presses a Save button, ensuring the validity of the data before saving it.

You can validate any window, modal or modeless, at any time. Only modal windows have automatic validation on closing, however. If you use modeless data entry windows, you need to ensure that your application validates the window before acting on entered data.

## Using a Data Validator

Using a data validator object with an input line takes only two simple steps:

1. Constructing the validator object
2. Assigning the validator to an input line

Once you've constructed the validator and associated it with an input line, you never need to interact with the validator object directly. The input line knows when to call validator methods at the appropriate times.

### Constructing Validator Objects

Since validators are not views, their constructors require only enough information to establish the validation criteria. For example, a numeric range validator takes two parameters: the minimum and maximum values in the valid range.

```rust
use turbo_vision::views::validator::RangeValidator;

// Create a validator for values between 100 and 999
let validator = RangeValidator::new(100, 999);
```

### Adding Validation to Input Lines

Every input line object can have an associated validator. If you don't assign a validator to an input line, the input line behaves as described in Chapter 12, "Control Objects," accepting any input. Once you assign a validator, the input line automatically checks with the validator when processing key events and when called on to validate itself.

Normally you construct and assign the validator in a single operation:

```rust
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::views::validator::RangeValidator;
use turbo_vision::core::geometry::Rect;
use std::rc::Rc;
use std::cell::RefCell;

// Create shared data
let data = Rc::new(RefCell::new(String::new()));

// Create input line with validator using the builder pattern
let validator = Rc::new(RefCell::new(RangeValidator::new(100, 999)));
let input_line = InputLineBuilder::new()
    .bounds(Rect::new(5, 2, 15, 3))
    .max_length(3)
    .data(data)
    .validator(validator)
    .build();
```

You can also set or change the validator after creation:

```rust
input_line.set_validator(validator);
```

## How Validators Work

Turbo Vision supplies several kinds of validator objects that cover most data validation needs. You can also create your own validators by implementing the `Validator` trait.

This section covers the following topics:

- The methods of a validator
- The standard validator types

### The Methods of a Validator

Every validator object implements the `Validator` trait (see `src/views/validator.rs`). This trait defines four important methods that validators use to perform their specific validation tasks. If you're going to modify the standard validators or write your own validation objects, you need to understand what each of these methods does and how input lines use them.

The four validation methods are:

- `valid` - Check if input is valid and show error if not
- `is_valid` - Check if the complete input is valid
- `is_valid_input` - Check if input is valid during typing
- `error` - Display error message

The only methods called from outside the object are `valid` and `is_valid_input`. The `error` and `is_valid` methods are only called by other validator methods.

#### Checking for Valid Data

The main external interface to data validator objects is the method `valid`. Like the view method of the same name, `valid` is a boolean function that returns true only if the string passed to it is valid data. One component of an input line's validation is calling the validator's `valid` method, passing the input line's current text.

When using validators with input lines, you should never need to either call or override the validator's `valid` method. By default, `valid` returns true if the method `is_valid` returns true; otherwise it calls `error` to notify the user of the error and returns false.

```rust
// Default implementation from the Validator trait
fn valid(&self, input: &str) -> bool {
    if self.is_valid(input) {
        true
    } else {
        self.error();
        false
    }
}
```

#### Validating a Complete Line

Validator objects have a method called `is_valid` that takes a string as its only parameter and returns true if the string represents valid data. `is_valid` is the method that does the actual validation, so if you create your own validator objects, you'll override `is_valid` in most cases.

You don't call `is_valid` directly. Use `valid` to call `is_valid`, because `valid` calls `error` to alert the user if `is_valid` returns false. Be sure to keep the validation role separate from the error reporting role.

```rust
use turbo_vision::views::validator::Validator;

// Example: custom validator implementation
struct MyValidator;

impl Validator for MyValidator {
    fn is_valid(&self, input: &str) -> bool {
        // Perform your validation logic here
        input.len() >= 3
    }

    fn error(&self) {
        // Display error message
    }
}
```

#### Validating Keystrokes

When an input line object recognizes a keystroke event meant for it, it calls its validator's `is_valid_input` method to ensure that the typed character is a valid entry. By default, `is_valid_input` methods always return true, meaning that all keystrokes are acceptable. However, some validators override `is_valid_input` to filter out unwanted keystrokes.

For example, range validators, which are used for numeric input, return true from `is_valid_input` only for numeric digits and the characters '+' and '-'.

`is_valid_input` takes two parameters. The first parameter holds the current input text. The second parameter is a boolean value indicating whether the validator should apply filling or padding to the input string before attempting to validate it. `PictureValidator` is the only one of the standard validator objects that makes significant use of the second parameter.

```rust
fn is_valid_input(&self, input: &str, append: bool) -> bool {
    // Check if the current input (during typing) is valid
    // Return false to reject the character
    input.chars().all(|c| c.is_ascii_digit())
}
```

#### Reporting Invalid Data

The method `error` alerts the user that the contents of the input line don't pass the validation check. The standard validator objects generally present a simple message (or could present a message box) notifying the user that the contents of the input are invalid and describing what proper input would be.

For example, the `error` method for a range validator object would indicate that the value in the input line is not between the indicated minimum and maximum values.

Although most validators override `error`, you should never call it directly. `valid` calls `error` for you if `is_valid` returns false, which is the only time `error` needs to be called.

## The Standard Validators

Turbo Vision includes several standard validator types, including an abstract validator trait and the following specific validator types:

- Filter validator
- Range validator
- Lookup validator
- Picture validator

### The Abstract Validator

The abstract `Validator` trait serves as the base for all validator objects, but does nothing useful by itself. Essentially, a default validator is one to which all input is always valid. `is_valid` and `is_valid_input` always return true by default, and `error` does nothing. Concrete types override `is_valid` and/or `is_valid_input` to define which values actually are valid.

You can use the `Validator` trait as a starting point for your own validator objects if none of the other validation types are appropriate starting points.

### Filter Validators

Filter validators (see `src/views/validator.rs`) are a simple implementation of validators that only check input as the user types it. The filter validator constructor takes one parameter, a string of valid characters:

```rust
use turbo_vision::views::validator::FilterValidator;

// Allow only digits
let validator = FilterValidator::new("0123456789");

// Allow alphanumeric characters
let validator = FilterValidator::new(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
);

// Allow hexadecimal digits
let validator = FilterValidator::new("0123456789ABCDEFabcdef");
```

`FilterValidator` overrides `is_valid_input` to return true only if all characters in the current input string are contained in the set of characters passed to the constructor. The input line only inserts characters if `is_valid_input` returns true, so there is no need to override `is_valid`. Because the characters made it through the input filter, the complete string is valid by definition.

Example usage:

```rust
use turbo_vision::views::validator::FilterValidator;
use turbo_vision::views::input_line::InputLineBuilder;
use std::rc::Rc;
use std::cell::RefCell;

// Create a filter validator for digits only
let validator = Rc::new(RefCell::new(
    FilterValidator::new("0123456789")
));

// Attach to an input line using the builder pattern
let input_line = InputLineBuilder::new()
    .bounds(Rect::new(5, 2, 15, 3))
    .max_length(10)
    .data(data)
    .validator(validator)
    .build();

// User can only type digits - other characters are rejected
```

Descendants of `FilterValidator`, such as `RangeValidator`, can combine filtering of input with other checks on the completed string.

### Range Validators

The range validator `RangeValidator` is a straightforward descendant concept of `FilterValidator` that accepts only numbers and adds range checking on the final results. The constructor takes two parameters that define the minimum and maximum valid values:

```rust
use turbo_vision::views::validator::RangeValidator;

// Create a validator for values between 100 and 999
let validator = RangeValidator::new(100, 999);

// Create a validator for negative values
let validator = RangeValidator::new(-100, -1);

// Create a validator for mixed positive and negative
let validator = RangeValidator::new(-50, 50);
```

The range validator works as a numeric filter validator, accepting only the digits '0'..'9' and the plus and minus characters. The inherited `is_valid_input` therefore ensures that only valid numeric characters filter through. `RangeValidator` then overrides `is_valid` to return true only if the entered numbers are a valid integer within the range defined in the constructor.

The range validator supports multiple numeric formats:

- **Decimal**: Regular numbers like `123`, `-45`, `+67`
- **Hexadecimal**: Numbers with `0x` prefix like `0xFF`, `0x1A`
- **Octal**: Numbers with `0` prefix like `077`, `0100`

Example usage:

```rust
use turbo_vision::views::validator::RangeValidator;
use turbo_vision::views::input_line::InputLineBuilder;
use std::rc::Rc;
use std::cell::RefCell;

// Create a validator for percentages (0-100)
let validator = Rc::new(RefCell::new(
    RangeValidator::new(0, 100)
));

// Attach to input line using the builder pattern
let input_line = InputLineBuilder::new()
    .bounds(Rect::new(5, 2, 15, 3))
    .max_length(3)
    .data(data)
    .validator(validator)
    .build();

// User can type: 0, 50, 100 (valid)
// User cannot enter: -1, 101, abc (invalid)
```

The `error` method would display a message indicating that the entered value is out of range.

### Lookup Validators

The lookup validator `LookupValidator` (see `src/views/lookup_validator.rs`) provides validation by comparing the entered value with a list of acceptable items.

The lookup validator compares the string passed from the input line with items in a list. If the passed string occurs in the list, the validation succeeds. The constructor takes a vector of valid strings:

```rust
use turbo_vision::views::lookup_validator::LookupValidator;

// Create a lookup validator with valid values
let validator = LookupValidator::new(vec![
    "Red".to_string(),
    "Green".to_string(),
    "Blue".to_string(),
]);
```

You can also create a case-insensitive lookup validator:

```rust
// Case-insensitive validator
let validator = LookupValidator::new_case_insensitive(vec![
    "Red".to_string(),
    "Green".to_string(),
    "Blue".to_string(),
]);

// Now "red", "RED", "Red" all validate
```

The lookup validator provides methods to modify the list of valid values:

```rust
// Add a valid value
validator.add_value("Yellow".to_string());

// Remove a valid value
validator.remove_value("Blue");

// Check if a value is valid
if validator.contains("Red") {
    // "Red" is in the valid list
}

// Change case sensitivity
validator.set_case_sensitive(false);
```

`LookupValidator` overrides `is_valid` to return true only if the input string is in the list of valid values. The `error` method would display a message indicating that the string wasn't in the list.

Example usage:

```rust
use turbo_vision::views::lookup_validator::LookupValidator;
use turbo_vision::views::input_line::InputLineBuilder;
use std::rc::Rc;
use std::cell::RefCell;

// Create validator for color selection
let validator = Rc::new(RefCell::new(
    LookupValidator::new_case_insensitive(vec![
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Yellow".to_string(),
    ])
));

// Attach to input line using the builder pattern
let input_line = InputLineBuilder::new()
    .bounds(Rect::new(5, 2, 20, 3))
    .max_length(20)
    .data(data)
    .validator(validator)
    .build();

// User can type: "Red", "green", "BLUE" (all valid)
// User cannot finalize: "Purple", "Orange" (invalid)
```

### Picture Validators

Picture validators (see `src/views/picture_validator.rs`) compare the string typed by the user with a picture or template that describes the format of valid input. The pictures used are compatible with those used by Borland's Paradox relational database to control user input.

Constructing a picture validator takes one parameter: a string holding the template image:

```rust
use turbo_vision::views::picture_validator::PictureValidator;

// Phone number format
let validator = PictureValidator::new("(###) ###-####");

// Date format
let validator = PictureValidator::new("##/##/####");

// Product code format
let validator = PictureValidator::new("@@@@-####");
```

#### Picture Mask Characters

The picture mask uses special characters to define the format:

| Character | Meaning | Example |
|-----------|---------|---------|
| `#` | Digit (0-9) | `###` matches `123` |
| `@` | Alpha (A-Z, a-z) | `@@@@` matches `ABCD` |
| `!` | Any character | `!!!` matches `a1@` |
| `*` | Optional marker | `###*-####` allows optional dash |
| Literal | Must match exactly | `(`, `)`, `-`, `/`, etc. |

#### Picture Validator Examples

**Phone Number**:
```rust
let validator = PictureValidator::new("(###) ###-####");
// Accepts: "(555) 123-4567"
// Rejects: "555-123-4567"
```

**Date**:
```rust
let validator = PictureValidator::new("##/##/####");
// Accepts: "12/25/2023"
// Rejects: "12-25-2023"
```

**Product Code**:
```rust
let validator = PictureValidator::new("@@@@-####");
// Accepts: "ABCD-1234"
// Rejects: "1234-ABCD"
```

**Optional Section**:
```rust
let validator = PictureValidator::new("###*-####");
// Accepts: "123-4567" or "1234567"
// The '*' makes the dash optional
```

#### Picture Validator Features

`PictureValidator` provides several methods:

```rust
// Create validator with auto-formatting
let validator = PictureValidator::new("(###) ###-####");

// Create validator without auto-formatting
let validator = PictureValidator::new_no_format("##/##/####");

// Format input according to mask
let formatted = validator.format("5551234567");
// Returns: "(555) 123-4567"

// Get the mask
let mask = validator.mask();

// Enable/disable auto-formatting
validator.set_auto_format(true);
```

`PictureValidator` overrides `error`, `is_valid_input`, and `is_valid`:

- `error` displays a message indicating what format the string should have
- `is_valid` returns true only if the input matches the picture format completely
- `is_valid_input` checks characters as the user types them, allowing only those allowed by the picture format, and optionally filling in literal characters from the picture

Example usage:

```rust
use turbo_vision::views::picture_validator::PictureValidator;
use turbo_vision::views::input_line::InputLineBuilder;
use std::rc::Rc;
use std::cell::RefCell;

// Create a phone number validator
let validator = Rc::new(RefCell::new(
    PictureValidator::new("(###) ###-####")
));

// Attach to input line using the builder pattern
let input_line = InputLineBuilder::new()
    .bounds(Rect::new(5, 2, 20, 3))
    .max_length(14)
    .data(data)
    .validator(validator)
    .build();

// As user types "5551234567", it auto-formats to "(555) 123-4567"
// User can only type digits in # positions
// Literal characters ( ) - are inserted automatically
```

## Creating Custom Validators

To create a custom validator, implement the `Validator` trait:

```rust
use turbo_vision::views::validator::Validator;

/// Custom validator that requires input to start with a specific prefix
struct PrefixValidator {
    prefix: String,
}

impl PrefixValidator {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

impl Validator for PrefixValidator {
    fn is_valid(&self, input: &str) -> bool {
        if input.is_empty() {
            return true; // Empty is valid
        }
        input.starts_with(&self.prefix)
    }

    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        // Allow typing if it could eventually match
        if input.is_empty() {
            return true;
        }

        // Check if input is a valid prefix of the required prefix
        // or starts with the complete prefix
        self.prefix.starts_with(input) || input.starts_with(&self.prefix)
    }

    fn error(&self) {
        eprintln!("Input must start with '{}'", self.prefix);
    }
}

// Usage
let validator = Rc::new(RefCell::new(
    PrefixValidator::new("ID-")
));

// Accepts: "ID-123", "ID-ABC"
// Rejects: "123", "ABC"
```

## Summary

Data validation in Turbo Vision provides a flexible and powerful system for ensuring data integrity:

- **Filter validators** restrict which characters can be typed
- **Range validators** ensure numeric values fall within a specified range
- **Lookup validators** ensure values match a predefined list
- **Picture validators** enforce specific formatting patterns

All validators implement the `Validator` trait, which defines:
- `is_valid()` - Validates the complete input
- `is_valid_input()` - Validates during typing (character filtering)
- `error()` - Displays error messages
- `valid()` - Combines validation with error reporting

Validators are attached to input lines using `InputLine::with_validator()` or `set_validator()`. Once attached, the input line automatically uses the validator to filter keystrokes and validate data, requiring no additional application code.

The Rust implementations can be found in:
- `src/views/validator.rs` - Base trait and FilterValidator, RangeValidator
- `src/views/picture_validator.rs` - PictureValidator
- `src/views/lookup_validator.rs` - LookupValidator
