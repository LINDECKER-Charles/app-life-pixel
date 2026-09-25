//! What the stack tests share: a hosted store over a test database and a test bucket prefix, two
//! accounts, and object storage whose deletions can be made to fail.

use std::sync::Arc;

use life_pixel_server::storage::HostedLibraryStore;
use life_pixel_server::testing::{TestDatabase, TestStorage};
use life_pixel_service::testing::StoreFixture;
use life_pixel_service::{AccountId, Owner};
use object_store::ObjectStore;

use crate::failing_objects::FailingDeletes;

/// A hosted store on the local stack, and what it stands on.
pub struct Stack {
    pub database: TestDatabase,
    pub storage: TestStorage,
    pub objects: Arc<FailingDeletes>,
    pub store: Arc<HostedLibraryStore>,
    pub account: AccountId,
}

impl Stack {
    /// A new database and bucket prefix, one account.
    pub async fn new() -> Self {
        let database = TestDatabase::create().await.unwrap();
        let storage = TestStorage::create().unwrap();
        let account = database.create_account().await.unwrap();
        let objects = Arc::new(FailingDeletes::new(storage.objects()));
        let dyn_objects: Arc<dyn ObjectStore> = objects.clone();
        let store = HostedLibraryStore::new(database.pool().clone(), dyn_objects);
        Self {
            database,
            storage,
            objects,
            store: Arc::new(store),
            account,
        }
    }

    /// The account's owner.
    pub fn owner(&self) -> Owner {
        Owner::Account(self.account)
    }

    /// The account's usage, as its row says.
    pub async fn stored_usage(&self) -> i64 {
        sqlx::query_scalar("select storage_used_bytes from accounts where id = $1")
            .bind(self.account.uuid())
            .fetch_one(self.database.pool())
            .await
            .unwrap()
    }

    /// The bytes the account's rows point at: what its usage must be.
    pub async fn referenced_bytes(&self) -> i64 {
        let query = "select coalesce(sum(document_bytes), 0)::bigint from animations \
                     where account_id = $1";
        sqlx::query_scalar(query)
            .bind(self.account.uuid())
            .fetch_one(self.database.pool())
            .await
            .unwrap()
    }
}

/// The contract's fixture over the hosted store on the local stack: two accounts of a new
/// database, and a new bucket prefix.
pub async fn contract_fixture() -> StoreFixture {
    let database = TestDatabase::create().await.unwrap();
    let storage = TestStorage::create().unwrap();
    let store = HostedLibraryStore::new(database.pool().clone(), storage.objects());
    fixture_for(store, database, storage).await
}

/// The fixture acting for two new accounts of `database`, keeping `guard` alive.
pub async fn fixture_for(
    store: HostedLibraryStore,
    database: TestDatabase,
    guard: impl std::any::Any + Send + Sync,
) -> StoreFixture {
    let owner = Owner::Account(database.create_account().await.unwrap());
    let other_owner = Owner::Account(database.create_account().await.unwrap());
    StoreFixture::new(store)
        .with_owners(owner, other_owner)
        .with_guard((database, guard))
}
