//! The internal admin API's use cases (H10), as docs/admin-console.md's "Administration" and
//! "Support requests" describe them: the users — search, profile, activity, suspension,
//! reactivation, data export and erasure —, the support queue — status, assignment, internal
//! notes, replies emailed and shown in the app —, and the product metrics.
//!
//! [`Admin`] holds the use cases, over four ports of their own — [`AdminUserStore`],
//! [`AdminSupportStore`], [`AuditLog`] and [`MetricsSource`] —, the accounts and the library of
//! H5 and H6, the screenshots of H9, the mailer and the clock. Every action names the admin who
//! acts ([`AdminIdentity`]) and lands in the append-only audit log ([`AuditEntry`]): a change
//! writes its entry in its own transaction, so that neither exists without the other, and reading
//! an account's data export writes one too. No entry holds a person's address or a message's
//! text.
//!
//! [`AdminUserStore`]: ports::AdminUserStore
//! [`AdminSupportStore`]: ports::AdminSupportStore
//! [`AuditLog`]: ports::AuditLog
//! [`MetricsSource`]: ports::MetricsSource

mod audit;
mod cases;
mod error;
pub mod ports;
mod values;

#[cfg(feature = "testing")]
pub mod memory;

use std::sync::Arc;

use time::{Duration, OffsetDateTime};

pub use audit::{
    AuditAction, AuditDraft, AuditEntry, AuditFilter, AuditRecord, SUPPORT_REQUEST_TARGET,
    USER_TARGET,
};
pub use cases::{MessageDraft, ProductMetrics, QueuedRequest, QueuedThread, RequestUpdate};
pub use error::AdminError;
pub use values::{
    ADMIN_EMAIL_MAX_CHARS, ADMIN_ID_MAX_CHARS, AdminIdentity, Assignee, REASON_MAX_CHARS,
    REASON_MIN_CHARS, Reason,
};

use self::ports::{AdminSupportStore, AdminUserStore, AuditLog, MetricsSource};
use crate::accounts::Accounts;
use crate::accounts::ports::Mailer;
use crate::library::Library;
use crate::ports::{Clock, IdGenerator};
use crate::support::ports::ScreenshotStore;

/// How far back an account's detail counts its product events.
pub const USER_EVENTS_WINDOW: Duration = Duration::days(30);
/// The period of the product metrics when none is asked for: the last 30 days.
pub const DEFAULT_METRICS_PERIOD: Duration = Duration::days(30);

/// The stores of the admin use cases.
#[derive(Clone)]
pub struct AdminStores {
    /// Where accounts are read and acted on.
    pub users: Arc<dyn AdminUserStore>,
    /// Where support requests are read and answered.
    pub support: Arc<dyn AdminSupportStore>,
    /// The audit log.
    pub audit: Arc<dyn AuditLog>,
    /// The product metrics.
    pub metrics: Arc<dyn MetricsSource>,
}

/// What the admin use cases work through.
#[derive(Clone)]
pub struct AdminPorts {
    /// Their own stores.
    pub stores: AdminStores,
    /// The accounts, for their data export (H6).
    pub accounts: Accounts,
    /// The hosted library, for the export and the erasure (H6).
    pub library: Library,
    /// The screenshots of support requests (H9).
    pub screenshots: Arc<dyn ScreenshotStore>,
    /// How replies are emailed.
    pub mailer: Arc<dyn Mailer>,
    /// The time of actions.
    pub clock: Arc<dyn Clock>,
    /// New message ids.
    pub ids: Arc<dyn IdGenerator>,
}

/// What configuration decides for the admin use cases.
#[derive(Clone, Debug)]
pub struct AdminSettings {
    /// The origin the links of reply emails start with: `LP_PUBLIC_URL`.
    pub public_url: String,
}

/// The admin use cases.
#[derive(Clone)]
pub struct Admin {
    ports: AdminPorts,
    settings: Arc<AdminSettings>,
}

impl Admin {
    /// The use cases over `ports`, with the values of configuration.
    #[must_use]
    pub fn new(ports: AdminPorts, settings: AdminSettings) -> Self {
        Self {
            ports,
            settings: Arc::new(settings),
        }
    }

    fn now(&self) -> OffsetDateTime {
        self.ports.clock.now()
    }

    fn users(&self) -> &dyn AdminUserStore {
        self.ports.stores.users.as_ref()
    }

    fn support(&self) -> &dyn AdminSupportStore {
        self.ports.stores.support.as_ref()
    }

    fn audit(&self) -> &dyn AuditLog {
        self.ports.stores.audit.as_ref()
    }

    /// The draft of `action` by `admin` on `target_id`, now.
    fn draft(
        &self,
        admin: &AdminIdentity,
        (action, target_id): (AuditAction, String),
    ) -> AuditDraft {
        AuditDraft {
            admin: admin.clone(),
            action,
            target_id,
            reason: None,
            at: self.now(),
        }
    }
}
