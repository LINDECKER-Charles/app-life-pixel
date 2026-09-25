//! The library store's contract suite on the in-memory adapter.

use life_pixel_service::memory::InMemoryLibraryStore;
use life_pixel_service::testing::library_store_contract;

async fn in_memory_store() -> InMemoryLibraryStore {
    InMemoryLibraryStore::new()
}

library_store_contract!(in_memory_store);
