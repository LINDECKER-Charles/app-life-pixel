//! What the team changes on a request, and how: the rules the store applies under its lock, and
//! the state its audit entries keep.

use serde_json::{Value, json};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::admin::Assignee;
use crate::support::SupportStatus;

/// The part of a request the team changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestState {
    /// Where it stands.
    pub status: SupportStatus,
    /// The admin it is assigned to.
    pub assigned_to: Option<String>,
    /// When the team first replied.
    pub first_response_at: Option<OffsetDateTime>,
    /// When it was resolved or closed, while it is.
    pub resolved_at: Option<OffsetDateTime>,
}

impl RequestState {
    /// The state once its status becomes `status`, `at`: a request resolved or closed keeps the
    /// time it first was; one opened again loses it.
    #[must_use]
    pub fn with_status(self, status: SupportStatus, at: OffsetDateTime) -> Self {
        let is_settled = matches!(status, SupportStatus::Resolved | SupportStatus::Closed);
        let resolved_at = if is_settled {
            self.resolved_at.or(Some(at))
        } else {
            None
        };
        Self {
            status,
            resolved_at,
            ..self
        }
    }

    /// The state once assigned to `assignee`, or to nobody.
    #[must_use]
    pub fn with_assignee(self, assignee: Option<&Assignee>) -> Self {
        Self {
            assigned_to: assignee.map(|assignee| assignee.as_str().to_owned()),
            ..self
        }
    }

    /// The state once the team replies, `at`: waiting for the person, answered from the first
    /// reply on.
    #[must_use]
    pub fn answered(self, at: OffsetDateTime) -> Self {
        let first_response_at = self.first_response_at.or(Some(at));
        Self {
            first_response_at,
            ..self.with_status(SupportStatus::WaitingForUser, at)
        }
    }

    /// The state as an audit entry keeps it.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "status": self.status.as_str(),
            "assignedTo": self.assigned_to,
            "firstResponseAt": self.first_response_at.map(rfc3339),
            "resolvedAt": self.resolved_at.map(rfc3339),
        })
    }
}

fn rfc3339(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    const EARLIER: OffsetDateTime = datetime!(2026-09-01 10:00 UTC);
    const NOW: OffsetDateTime = datetime!(2026-09-02 10:00 UTC);

    fn state(status: SupportStatus) -> RequestState {
        RequestState {
            status,
            assigned_to: None,
            first_response_at: None,
            resolved_at: None,
        }
    }

    #[test]
    fn resolving_keeps_the_first_resolution_and_reopening_forgets_it() {
        let resolved =
            state(SupportStatus::InProgress).with_status(SupportStatus::Resolved, EARLIER);
        assert_eq!(resolved.resolved_at, Some(EARLIER));
        let closed = resolved.with_status(SupportStatus::Closed, NOW);
        assert_eq!(closed.resolved_at, Some(EARLIER));
        let reopened = closed.with_status(SupportStatus::InProgress, NOW);
        assert_eq!(reopened.resolved_at, None);
    }

    #[test]
    fn a_reply_waits_for_the_person_and_keeps_the_first_response() {
        let answered = state(SupportStatus::New).answered(EARLIER);
        assert_eq!(answered.status, SupportStatus::WaitingForUser);
        assert_eq!(answered.first_response_at, Some(EARLIER));
        let again = answered.answered(NOW);
        assert_eq!(again.first_response_at, Some(EARLIER));
    }
}
