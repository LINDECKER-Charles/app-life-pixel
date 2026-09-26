//! Reading the users: the search, and one user's profile and activity.

use crate::admin::ports::{UserDetail, UserFilter, UserSummary};
use crate::admin::{Admin, AdminError, USER_EVENTS_WINDOW};
use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};

impl Admin {
    /// A page of the users `filter` keeps, from the most recent sign-up.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn search_users(
        &self,
        filter: UserFilter,
        page: PageRequest,
    ) -> Result<Page<UserSummary>, AdminError> {
        Ok(self.users().search(filter, page).await?)
    }

    /// The user `id`: the profile, the counts of the library, the product events of the last
    /// [`USER_EVENTS_WINDOW`] by name, the sessions and the support requests.
    ///
    /// # Errors
    ///
    /// `admin.user_not_found`, `service.unavailable`.
    pub async fn user(&self, id: AccountId) -> Result<UserDetail, AdminError> {
        let since = self.now() - USER_EVENTS_WINDOW;
        let detail = self.users().detail(id, since).await?;
        detail.ok_or(AdminError::UserNotFound)
    }
}
