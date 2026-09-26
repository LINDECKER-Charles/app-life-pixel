//! The daily ceiling, the signed links and the downloads over the in-memory adapters.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

use std::sync::Arc;

use life_pixel_compiler::ExportFormat;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::memory::{InMemoryAnimationOwners, InMemoryMcpUsage};
use super::{DailyCeiling, ExportDownloads, ExportLink, ExportLinks, McpError};
use crate::animation::{AnimationEditing, ExportRequest};
use crate::ids::{AccountId, AnimationId};
use crate::library::{Library, LibraryPorts};
use crate::memory::{FixedClock, InMemoryLibraryStore, RecordingEvents, SequentialIds};
use crate::testing::sample_document;
use crate::{Coded, Owner, Plans};

/// Late in a UTC day.
const START: OffsetDateTime = datetime!(2026-09-01 23:59 UTC);
const KEY: [u8; 32] = [7; 32];

fn account() -> AccountId {
    AccountId::from_uuid(Uuid::from_u128(0xacc0))
}

fn plans(free_mcp_calls_per_day: u32) -> Plans {
    Plans {
        free_storage_bytes: 1_000_000,
        free_mcp_calls_per_day,
    }
}

#[tokio::test]
async fn the_daily_ceiling_refuses_beyond_the_plan_until_midnight_utc() {
    let usage = Arc::new(InMemoryMcpUsage::new());
    let clock = Arc::new(FixedClock::new(START));
    let ceiling = DailyCeiling::new(usage.clone(), clock.clone(), plans(2));

    ceiling.count_call(account()).await.unwrap();
    ceiling.count_call(account()).await.unwrap();
    let refused = ceiling.count_call(account()).await.unwrap_err();

    assert_eq!(refused.code(), "mcp.daily_limit");
    assert_eq!(refused.params()["limit"], 2);
    assert_eq!(refused.params()["resetsAt"], "2026-09-02T00:00:00Z");
    assert_eq!(usage.calls(account(), START.date()), 2);
    clock.advance(Duration::minutes(1));
    assert!(ceiling.count_call(account()).await.is_ok());
}

fn link(links: &ExportLinks, animation: AnimationId, version: u64) -> ExportLink {
    ExportLink {
        animation,
        version,
        format: ExportFormat::Gif,
        tag: None,
        scale: Some(2),
        file_name: "mascot.gif".to_owned(),
        expires_at: links.expiry(),
    }
}

#[test]
fn a_link_verifies_until_it_expires_and_never_once_tampered_with() {
    let clock = Arc::new(FixedClock::new(START));
    let links = ExportLinks::new(&KEY, clock.clone());
    let expected = link(&links, AnimationId::from_uuid(Uuid::nil()), 3);
    let text = links.sign(&expected);

    assert_eq!(links.verify(&text).unwrap(), expected);
    let (payload, signature) = text.split_once('.').unwrap();
    let mut forged = expected.clone();
    forged.version = 4;
    let (forged_payload, _) = links
        .sign(&forged)
        .split_once('.')
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .unwrap();
    let other_key = ExportLinks::new(&[8; 32], clock.clone()).sign(&expected);
    let tampered = [
        format!("{forged_payload}.{signature}"),
        format!("{payload}.{}", &signature[1..]),
        format!("{payload}{signature}"),
        other_key,
        String::new(),
    ];
    for text in tampered {
        assert_eq!(links.verify(&text), Err(McpError::LinkInvalid), "{text}");
    }
    clock.advance(Duration::minutes(15));
    assert_eq!(links.verify(&text), Err(McpError::LinkInvalid));
}

struct Downloads {
    downloads: ExportDownloads,
    library: Library,
    links: ExportLinks,
    animation: AnimationId,
    version: u64,
}

async fn downloads() -> Downloads {
    let clock = Arc::new(FixedClock::new(START));
    let events = Arc::new(RecordingEvents::new());
    let ports = LibraryPorts {
        store: Arc::new(InMemoryLibraryStore::new()),
        clock: clock.clone(),
        ids: Arc::new(SequentialIds::new()),
        events: events.clone(),
    };
    let library = Library::new(ports, plans(10));
    let owner = Owner::Account(account());
    let project = library.create_project(&owner, "Game").await.unwrap();
    let (_, document) = sample_document("Mascot");
    let record = library.import_animation(&owner, project.id, document).await;
    let record = record.unwrap();
    let owners = Arc::new(InMemoryAnimationOwners::new());
    owners.insert(record.id, account());
    let links = ExportLinks::new(&KEY, clock);
    let editing = AnimationEditing::new(library.clone(), events);
    let parts = (library.clone(), editing);
    Downloads {
        downloads: ExportDownloads::new(links.clone(), owners, parts),
        library,
        links,
        animation: record.id,
        version: record.version,
    }
}

#[tokio::test]
async fn a_link_downloads_its_file_compiled_from_its_version() {
    let setup = downloads().await;
    let text = setup
        .links
        .sign(&link(&setup.links, setup.animation, setup.version));

    let file = setup.downloads.download(&text).await.unwrap();

    assert_eq!(file.name, "mascot.gif");
    assert_eq!(file.media_type, "image/gif");
    let owner = Owner::Account(account());
    let request = ExportRequest {
        id: setup.animation,
        format: ExportFormat::Gif,
        tag: None,
        scale: Some(2),
    };
    let editing = AnimationEditing::new(setup.library.clone(), Arc::new(RecordingEvents::new()));
    let expected = editing.export(&owner, request).await.unwrap();
    assert_eq!(file.bytes, expected[0].bytes);
}

#[tokio::test]
async fn a_link_to_a_changed_or_unknown_animation_or_file_is_invalid() {
    let setup = downloads().await;
    let owner = Owner::Account(account());
    let mut unknown = link(&setup.links, AnimationId::from_uuid(Uuid::nil()), 1);
    let mut other_file = link(&setup.links, setup.animation, setup.version);
    other_file.file_name = "other.gif".to_owned();
    let current = link(&setup.links, setup.animation, setup.version);
    let current = setup.links.sign(&current);
    let renamed = setup
        .library
        .rename_animation(&owner, setup.animation, setup.version, "Hero");
    renamed.await.unwrap();
    unknown.expires_at = setup.links.expiry();

    for link in [&unknown, &other_file] {
        let refused = setup.downloads.download(&setup.links.sign(link)).await;
        assert_eq!(refused.unwrap_err().code, "export.link_invalid");
    }
    let stale = setup.downloads.download(&current).await.unwrap_err();
    assert_eq!(stale.code, "export.link_invalid");
}
