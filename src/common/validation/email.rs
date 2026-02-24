use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailValidationError {
    Empty,
    InvalidFormat,
}

pub fn validate_email(s: &str) -> Result<(), EmailValidationError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(EmailValidationError::Empty);
    }
    if !EMAIL_RE.is_match(s) {
        return Err(EmailValidationError::InvalidFormat);
    }
    Ok(())
}

// Convenience bool for quick checks
pub fn is_valid_email(s: &str) -> bool {
    validate_email(s).is_ok()
}

pub fn validate_email_opt(email: &Option<String>) -> Result<(), EmailValidationError> {
    match email {
        None => Ok(()),
        Some(s) => validate_email(s),
    }
}