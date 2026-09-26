//! The product metrics: H13's aggregates over a period, and the support queue's measures.

use time::OffsetDateTime;

use crate::admin::ports::{Period, ProductAggregates, SupportAggregates};
use crate::admin::{Admin, AdminError, DEFAULT_METRICS_PERIOD};

/// The product metrics of a period.
#[derive(Clone, Debug, PartialEq)]
pub struct ProductMetrics {
    /// The period.
    pub period: Period,
    /// H13's aggregates over it.
    pub product: ProductAggregates,
    /// The support queue's measures.
    pub support: SupportAggregates,
}

impl Admin {
    /// The product metrics from `since` to `until`: until now, and over the
    /// [`DEFAULT_METRICS_PERIOD`] before its end, when they are not given.
    ///
    /// # Errors
    ///
    /// `request.malformed` when the period ends before it starts, `service.unavailable`.
    pub async fn product_metrics(
        &self,
        (since, until): (Option<OffsetDateTime>, Option<OffsetDateTime>),
    ) -> Result<ProductMetrics, AdminError> {
        let now = self.now();
        let until = until.unwrap_or(now);
        let since = since.unwrap_or(until - DEFAULT_METRICS_PERIOD);
        if since >= until {
            return Err(AdminError::Malformed);
        }
        let period = Period { since, until };
        let metrics = self.ports.stores.metrics.as_ref();
        Ok(ProductMetrics {
            period,
            product: metrics.product(period).await?,
            support: metrics.support(period, now).await?,
        })
    }
}
