//! Search on the local stack: `ilike` with its wildcards escaped, served by the trigram index.

use std::time::{Duration, Instant};

use crate::common::Stack;
use life_pixel_service::ProjectId;
use life_pixel_service::paging::PageRequest;
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::ports::library_store::AnimationFilter;
use life_pixel_service::testing::{new_animation, new_project};

/// Rows that do not match: with fewer, a sequential scan costs less than the index.
const NOISE_ROWS: i32 = 50_000;
/// How long the statistics of another connection may take to arrive.
const STATISTICS_DELAY: Duration = Duration::from_secs(30);

const INSERT_NOISE: &str = "insert into animations (id, account_id, project_id, title, width, \
    height, frame_count, document_key, document_bytes, version, created_at, updated_at) \
    select gen_random_uuid(), $1, $2, 'Walker ' || n, 4, 4, 1, 'noise/' || n, 0, 1, now(), now() \
    from generate_series(1, $3) as n";
const TRIGRAM_SCANS: &str = "select coalesce(sum(idx_scan), 0)::bigint from pg_stat_user_indexes \
                             where indexrelname = 'animations_title_trgm'";

async fn project(stack: &Stack) -> ProjectId {
    let project = new_project("Animals");
    stack
        .store
        .create_project(&stack.owner(), project.clone())
        .await
        .unwrap();
    project.id
}

/// Creates an animation of `project` for each of `titles`.
async fn animations(stack: &Stack, project: ProjectId, titles: &[&str]) {
    for title in titles {
        let new = new_animation(project, title);
        stack
            .store
            .create_animation(&stack.owner(), new, None)
            .await
            .unwrap();
    }
}

async fn search(stack: &Stack, query: &str) -> Vec<String> {
    let filter = AnimationFilter {
        project: None,
        query: Some(query.to_owned()),
    };
    let owner = stack.owner();
    let page = stack
        .store
        .list_animations(&owner, filter, PageRequest::default());
    let mut titles: Vec<String> = page
        .await
        .unwrap()
        .items
        .into_iter()
        .map(|animation| animation.meta.title.as_str().to_owned())
        .collect();
    titles.sort();
    titles
}

async fn trigram_scans(stack: &Stack) -> i64 {
    let mut connection = stack.database.pool().acquire().await.unwrap();
    sqlx::query("select pg_stat_clear_snapshot()")
        .execute(&mut *connection)
        .await
        .unwrap();
    sqlx::query_scalar(TRIGRAM_SCANS)
        .fetch_one(&mut *connection)
        .await
        .unwrap()
}

#[tokio::test]
async fn search_takes_wildcards_literally() {
    let stack = Stack::new().await;
    let project = project(&stack).await;
    let titles = ["100% cat", "1000 cats", "a_b", "axb", r"back\slash"];
    animations(&stack, project, &titles).await;

    assert_eq!(search(&stack, "%").await, ["100% cat"]);
    assert_eq!(search(&stack, "0%").await, ["100% cat"]);
    assert_eq!(search(&stack, "_").await, ["a_b"]);
    assert_eq!(search(&stack, r"\").await, [r"back\slash"]);
    assert_eq!(search(&stack, "CAT").await, ["100% cat", "1000 cats"]);
}

#[tokio::test]
async fn search_is_served_by_the_trigram_index() {
    let stack = Stack::new().await;
    let project = project(&stack).await;
    sqlx::query(INSERT_NOISE)
        .bind(stack.account.uuid())
        .bind(project.uuid())
        .bind(NOISE_ROWS)
        .execute(stack.database.pool())
        .await
        .unwrap();
    animations(&stack, project, &["Zebra crossing", "Striped zebra"]).await;
    sqlx::query("analyze animations")
        .execute(stack.database.pool())
        .await
        .unwrap();
    let before = trigram_scans(&stack).await;

    let found = search(&stack, "zebra").await;

    assert_eq!(found, ["Striped zebra", "Zebra crossing"]);
    let deadline = Instant::now() + STATISTICS_DELAY;
    while trigram_scans(&stack).await == before {
        assert!(
            Instant::now() < deadline,
            "the trigram index was not scanned"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
