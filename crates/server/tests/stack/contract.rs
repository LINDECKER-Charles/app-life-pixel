//! H1's library store contract against `HostedLibraryStore`: on the local stack's S3Mock, then
//! on a local folder, the self-hosting default. Each case gets a database of its own.

use crate::common::contract_fixture;
use life_pixel_service::testing::library_store_contract;

library_store_contract!(contract_fixture);

mod local_folder {
    use life_pixel_server::config::StorageConfig;
    use life_pixel_server::storage::{HostedLibraryStore, object_store};
    use life_pixel_server::testing::TestDatabase;
    use life_pixel_service::testing::{StoreFixture, library_store_contract};

    use crate::common::fixture_for;

    async fn folder_fixture() -> StoreFixture {
        let database = TestDatabase::create().await.unwrap();
        let folder = tempfile::tempdir().unwrap();
        let root = folder.path().join("objects");
        let objects = object_store(&StorageConfig::File { root }).unwrap();
        let store = HostedLibraryStore::new(database.pool().clone(), objects);
        fixture_for(store, database, folder).await
    }

    library_store_contract!(folder_fixture);
}
