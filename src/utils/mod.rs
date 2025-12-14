use std::fmt;

pub mod display;
pub mod shell_generate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifierError {
    /// Empty string
    Empty,
    /// Starts with a digit
    StartsWithDigit,
    /// Contains invalid character
    InvalidCharacter(char),
    /// Contains lowercase letters (when uppercase is required)
    ContainsLowercase,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierError::Empty => {
                write!(f, "Identifier cannot be empty")
            }
            IdentifierError::StartsWithDigit => {
                write!(
                    f,
                    "Identifier cannot start with a digit, must start with a letter or underscore"
                )
            }
            IdentifierError::InvalidCharacter(ch) => {
                write!(
                    f,
                    "Identifier contains invalid character '{}', only letters, digits, and underscores are allowed",
                    ch
                )
            }
            IdentifierError::ContainsLowercase => {
                write!(f, "Identifier must be all uppercase")
            }
        }
    }
}

impl std::error::Error for IdentifierError {}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to require all uppercase letters
    pub require_uppercase: bool,
    /// Whether to allow leading underscore
    pub allow_leading_underscore: bool,
    /// Additional allowed characters (besides letters, digits, and underscores)
    pub additional_allowed_chars: Vec<char>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            require_uppercase: false,
            allow_leading_underscore: true,
            additional_allowed_chars: Vec::new(),
        }
    }
}

impl ValidationConfig {
    /// Configuration for environment variables (strict mode - all uppercase)
    pub fn env_var_strict() -> Self {
        Self {
            require_uppercase: true,
            allow_leading_underscore: true,
            additional_allowed_chars: Vec::new(),
        }
    }

    /// Configuration for environment variables (relaxed mode)
    pub fn env_var_relaxed() -> Self {
        Self {
            require_uppercase: false,
            allow_leading_underscore: true,
            additional_allowed_chars: Vec::new(),
        }
    }

    /// Configuration for variable names (allows hyphens)
    pub fn variable_name() -> Self {
        Self {
            require_uppercase: false,
            allow_leading_underscore: true,
            additional_allowed_chars: vec!['-'],
        }
    }

    /// Configuration for constant names (all uppercase, no leading underscore)
    pub fn constant_name() -> Self {
        Self {
            require_uppercase: true,
            allow_leading_underscore: false,
            additional_allowed_chars: Vec::new(),
        }
    }
}

pub fn validate_identifier(
    identifier: &str,
    config: &ValidationConfig,
) -> Result<(), IdentifierError> {
    // 1. Check if empty
    if identifier.is_empty() {
        return Err(IdentifierError::Empty);
    }

    // 2. Check first character
    if let Some(first_char) = identifier.chars().next() {
        // Cannot start with a digit
        if first_char.is_ascii_digit() {
            return Err(IdentifierError::StartsWithDigit);
        }

        // Check if leading underscore is allowed
        if first_char == '_' && !config.allow_leading_underscore {
            return Err(IdentifierError::InvalidCharacter(first_char));
        }

        // First character must be a letter or underscore
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(IdentifierError::InvalidCharacter(first_char));
        }
    }

    // 3. Check all characters
    for ch in identifier.chars() {
        // Check case requirements
        if config.require_uppercase && ch.is_ascii_lowercase() {
            return Err(IdentifierError::ContainsLowercase);
        }

        // Check if character is valid
        let is_valid = ch.is_ascii_alphanumeric()
            || ch == '_'
            || config.additional_allowed_chars.contains(&ch);

        if !is_valid {
            return Err(IdentifierError::InvalidCharacter(ch));
        }
    }

    Ok(())
}

pub fn validate_profile_name(name: &str) -> Result<(), IdentifierError> {
    validate_identifier(name, &ValidationConfig::variable_name())
}

pub fn validate_variable_key(key: &str) -> Result<(), IdentifierError> {
    validate_identifier(key, &ValidationConfig::variable_name())
}
