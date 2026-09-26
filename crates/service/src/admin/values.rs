//! The values of the admin use cases, each checked where it is made: the admin acting, the reason
//! of an action on an account, and the assignee of a request.

use crate::admin::AdminError;

/// The most characters of the acting admin's id, as the admin server sends it.
pub const ADMIN_ID_MAX_CHARS: usize = 100;
/// The most characters of an address: RFC 5321's limit.
pub const ADMIN_EMAIL_MAX_CHARS: usize = 254;
/// The fewest characters of a reason, once trimmed.
pub const REASON_MIN_CHARS: usize = 1;
/// The most characters of a reason, once trimmed.
pub const REASON_MAX_CHARS: usize = 1_000;

/// The admin acting, as the admin server vouches for them: what every audit entry names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminIdentity {
    id: String,
    email: String,
}

impl AdminIdentity {
    /// The admin of `id` and `email`, when both are present, short enough and printable, and the
    /// address has an `@`.
    #[must_use]
    pub fn parse(id: &str, email: &str) -> Option<Self> {
        let is_id = is_printable(id, ADMIN_ID_MAX_CHARS);
        let is_email = is_printable(email, ADMIN_EMAIL_MAX_CHARS) && email.contains('@');
        (is_id && is_email).then(|| Self {
            id: id.to_owned(),
            email: email.to_owned(),
        })
    }

    /// The admin's id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The admin's address.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
}

/// Whether `text` holds 1 to `max` characters, none of them a control or a space at either end.
fn is_printable(text: &str, max: usize) -> bool {
    let length = text.chars().count();
    (1..=max).contains(&length) && text.trim() == text && !text.chars().any(char::is_control)
}

/// Why an admin acted on an account: kept in the audit log, even after an erasure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reason(String);

impl Reason {
    /// The reason of `text`, without its surrounding whitespace.
    ///
    /// # Errors
    ///
    /// `admin.reason_length`, with `min` and `max`, when it is empty or too long.
    pub fn parse(text: &str) -> Result<Self, AdminError> {
        let trimmed = text.trim();
        let length = trimmed.chars().count();
        if !(REASON_MIN_CHARS..=REASON_MAX_CHARS).contains(&length) {
            return Err(AdminError::ReasonLength);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The admin a request is assigned to: an admin id, as [`AdminIdentity`] checks it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignee(String);

impl Assignee {
    /// The assignee of `id`.
    ///
    /// # Errors
    ///
    /// `request.malformed` when it is not an admin id.
    pub fn parse(id: &str) -> Result<Self, AdminError> {
        if !is_printable(id, ADMIN_ID_MAX_CHARS) {
            return Err(AdminError::Malformed);
        }
        Ok(Self(id.to_owned()))
    }

    /// The admin id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identity_needs_an_id_and_an_address() {
        assert!(AdminIdentity::parse("0190f6a2", "ada@example.org").is_some());
        for (id, email) in [
            ("", "ada@example.org"),
            ("0190f6a2", ""),
            ("0190f6a2", "not an address"),
            (" 0190f6a2", "ada@example.org"),
            ("0190\nf6a2", "ada@example.org"),
        ] {
            assert_eq!(AdminIdentity::parse(id, email), None, "{id:?} {email:?}");
        }
        let long = "a".repeat(ADMIN_ID_MAX_CHARS + 1);
        assert_eq!(AdminIdentity::parse(&long, "ada@example.org"), None);
    }

    #[test]
    fn a_reason_is_trimmed_and_bounded() {
        assert_eq!(Reason::parse("  spam  ").unwrap().as_str(), "spam");
        assert_eq!(Reason::parse("   "), Err(AdminError::ReasonLength));
        let long = "a".repeat(REASON_MAX_CHARS + 1);
        assert_eq!(Reason::parse(&long), Err(AdminError::ReasonLength));
    }
}
