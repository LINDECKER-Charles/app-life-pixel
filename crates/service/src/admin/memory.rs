//! Admin stores for the tests that never reach them: every call fails as a database that does not
//! answer would, `service.unavailable`. Feature `testing`.

use std::sync::Arc;

use async_trait::async_trait;
use time::OffsetDateTime;

use super::ports::{
    AdminRequest, AdminStoreError, AdminSupportStore, AdminThread, AdminUserStore, AuditLog,
    Erasure, MetricsSource, Period, ProductAggregates, QueueFilter, Recipient, RequestChange,
    StatusChange, SupportAggregates, TeamMessage, UserDetail, UserFilter, UserSummary,
};
use super::{AdminStores, AuditEntry, AuditFilter, AuditRecord};
use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};
use crate::support::SupportRequestId;

/// Stores that never answer.
struct Unavailable;

fn unavailable<T>() -> Result<T, AdminStoreError> {
    Err(AdminStoreError("no admin store in this test".to_owned()))
}

/// Admin stores whose every call fails, for the tests of other routes.
#[must_use]
pub fn unavailable_stores() -> AdminStores {
    AdminStores {
        users: Arc::new(Unavailable),
        support: Arc::new(Unavailable),
        audit: Arc::new(Unavailable),
        metrics: Arc::new(Unavailable),
    }
}

#[async_trait]
impl AdminUserStore for Unavailable {
    async fn search(
        &self,
        _: UserFilter,
        _: PageRequest,
    ) -> Result<Page<UserSummary>, AdminStoreError> {
        unavailable()
    }

    async fn find(&self, _: AccountId) -> Result<Option<UserSummary>, AdminStoreError> {
        unavailable()
    }

    async fn detail(
        &self,
        _: AccountId,
        _: OffsetDateTime,
    ) -> Result<Option<UserDetail>, AdminStoreError> {
        unavailable()
    }

    async fn set_status(&self, _: StatusChange) -> Result<bool, AdminStoreError> {
        unavailable()
    }

    async fn erase(&self, _: Erasure) -> Result<bool, AdminStoreError> {
        unavailable()
    }
}

#[async_trait]
impl AdminSupportStore for Unavailable {
    async fn queue(
        &self,
        _: QueueFilter,
        _: PageRequest,
    ) -> Result<Page<AdminRequest>, AdminStoreError> {
        unavailable()
    }

    async fn thread(&self, _: SupportRequestId) -> Result<Option<AdminThread>, AdminStoreError> {
        unavailable()
    }

    async fn screenshot_key(
        &self,
        _: SupportRequestId,
    ) -> Result<Option<Option<String>>, AdminStoreError> {
        unavailable()
    }

    async fn update(&self, _: RequestChange) -> Result<Option<AdminRequest>, AdminStoreError> {
        unavailable()
    }

    async fn add_team_message(&self, _: TeamMessage) -> Result<Option<Recipient>, AdminStoreError> {
        unavailable()
    }
}

#[async_trait]
impl AuditLog for Unavailable {
    async fn append(&self, _: AuditEntry) -> Result<(), AdminStoreError> {
        unavailable()
    }

    async fn list(
        &self,
        _: AuditFilter,
        _: PageRequest,
    ) -> Result<Page<AuditRecord>, AdminStoreError> {
        unavailable()
    }
}

#[async_trait]
impl MetricsSource for Unavailable {
    async fn product(&self, _: Period) -> Result<ProductAggregates, AdminStoreError> {
        unavailable()
    }

    async fn support(
        &self,
        _: Period,
        _: OffsetDateTime,
    ) -> Result<SupportAggregates, AdminStoreError> {
        unavailable()
    }
}
