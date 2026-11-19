use std::fmt;

/// A validated, normalized email address.
///
/// The only way to create one is [`Email::parse`], so any `Email` value
/// is proof that validation already happened.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

/// Why an input could not become an [`Email`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EmailError {
    #[error("email is empty")]
    Empty,
    #[error("email is longer than {max} characters")]
    TooLong { max: usize },
    #[error("email must look like name@example.com")]
    InvalidFormat,
}

impl Email {
    /// Maximum length of an email address (RFC 5321).
    pub const MAX_LEN: usize = 254;

    /// Validates and normalizes raw input into an [`Email`].
    ///
    /// Surrounding whitespace is trimmed and the address is lowercased.
    ///
    /// # Errors
    ///
    /// Returns an [`EmailError`] if the input is empty, too long,
    /// or not shaped like `local@domain.tld`.
    pub fn parse(input: &str) -> Result<Self, EmailError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(EmailError::Empty);
        }
        if trimmed.len() > Self::MAX_LEN {
            return Err(EmailError::TooLong { max: Self::MAX_LEN });
        }

        let (local, domain) = trimmed.split_once('@').ok_or(EmailError::InvalidFormat)?;
        if local.is_empty() || domain.contains('@') || !domain.contains('.') {
            return Err(EmailError::InvalidFormat);
        }

        Ok(Self(trimmed.to_lowercase()))
    }

    /// The email as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_valid_email() {
        let email = Email::parse("jane@example.com").unwrap();
        assert_eq!(email.as_str(), "jane@example.com");
    }

    #[test]
    fn trims_and_lowercases() {
        let email = Email::parse("  Jane@Example.COM ").unwrap();
        assert_eq!(email.as_str(), "jane@example.com");
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(Email::parse("   "), Err(EmailError::Empty));
    }

    #[test]
    fn rejects_too_long_input() {
        let input = format!("{}@example.com", "a".repeat(Email::MAX_LEN));
        assert_eq!(
            Email::parse(&input),
            Err(EmailError::TooLong {
                max: Email::MAX_LEN
            })
        );
    }

    #[test]
    fn rejects_malformed_input() {
        for input in ["jane", "@example.com", "jane@", "jane@example", "a@b@c.com"] {
            assert_eq!(
                Email::parse(input),
                Err(EmailError::InvalidFormat),
                "{input}"
            );
        }
    }
}
