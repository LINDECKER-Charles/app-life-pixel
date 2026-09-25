//! `auth_events_total{event}`: what happens to accounts, counted by event, never by account.

use metrics::{counter, describe_counter};

/// The accounts' events, by `event`.
pub const AUTH_EVENTS_TOTAL: &str = "auth_events_total";

/// An event of `auth_events_total`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthEvent {
    /// An account was created.
    SignUp,
    /// A session was opened with a password.
    SignIn,
    /// A sign-in was refused: wrong credentials, or a suspended account.
    SignInFailed,
    /// A session was ended by its owner.
    SignOut,
    /// A password was reset through an emailed link.
    PasswordReset,
    /// A password was changed by its owner.
    PasswordChanged,
}

impl AuthEvent {
    /// The value of the `event` label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::SignUp => "sign_up",
            Self::SignIn => "sign_in",
            Self::SignInFailed => "sign_in_failed",
            Self::SignOut => "sign_out",
            Self::PasswordReset => "password_reset",
            Self::PasswordChanged => "password_changed",
        }
    }
}

/// Describes the accounts' metrics to the recorder, once it is installed.
pub fn describe() {
    describe_counter!(
        AUTH_EVENTS_TOTAL,
        "Accounts' events: sign-ups, sign-ins and the like"
    );
}

/// Counts one `event`.
pub fn count(event: AuthEvent) {
    counter!(AUTH_EVENTS_TOTAL, "event" => event.label()).increment(1);
}
