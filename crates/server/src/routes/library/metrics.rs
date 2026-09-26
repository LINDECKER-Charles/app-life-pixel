//! `quota_rejections_total{kind}`: the changes a quota refused, by kind — `storage` here.

use metrics::{counter, describe_counter};

/// The changes a quota refused, by `kind`.
pub const QUOTA_REJECTIONS_TOTAL: &str = "quota_rejections_total";
/// The `kind` of the storage quota.
pub const STORAGE_KIND: &str = "storage";

/// Describes the quota metrics to the recorder, once it is installed.
pub fn describe() {
    describe_counter!(QUOTA_REJECTIONS_TOTAL, "Changes a quota refused, by kind");
}

/// Counts one change the storage quota refused.
pub fn count_storage_rejection() {
    counter!(QUOTA_REJECTIONS_TOTAL, "kind" => STORAGE_KIND).increment(1);
}
