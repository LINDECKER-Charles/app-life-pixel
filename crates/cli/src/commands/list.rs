//! `list`: the library's animations, most recently updated first, as a table or JSON —
//! `docs/v1/mcp-cli.md`'s "A4 — CLI".

use life_pixel_service::ports::library_store::{AnimationFilter, AnimationRecord};
use life_pixel_service::{CodedError, Owner, PageRequest, ProjectId};
use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::args::ListArgs;
use crate::messages::Messages;
use crate::{AppError, library};

/// The most animations fetched in one page while listing every one.
const PAGE_SIZE: u16 = 50;

/// Runs `list`: prints a table or, with `--json`, an array of animations.
///
/// # Errors
///
/// The library cannot be opened, or a page cannot be listed.
pub async fn run(args: ListArgs, messages: &Messages) -> Result<(), AppError> {
    let path = library::resolve_path(args.library);
    let (library, _editing) = library::open(&path)?;
    let filter = AnimationFilter {
        project: args.project.map(ProjectId::from_uuid),
        query: args.query,
    };
    let animations = all_animations(&library, filter).await?;
    if args.json {
        print_json(&animations);
    } else {
        print_table(&animations, messages);
    }
    Ok(())
}

/// Every page of `filter`'s animations, gathered in list order.
async fn all_animations(
    library: &life_pixel_service::library::Library,
    filter: AnimationFilter,
) -> Result<Vec<AnimationRecord>, AppError> {
    let mut page = PageRequest::new(None, Some(PAGE_SIZE));
    let mut animations = Vec::new();
    loop {
        let fetched = library.list_animations(&Owner::Local, filter.clone(), page);
        let fetched = fetched.await.map_err(|error| CodedError::of(&error))?;
        let next_cursor = fetched.next_cursor;
        animations.extend(fetched.items);
        let Some(cursor) = next_cursor else {
            break;
        };
        page = PageRequest::new(Some(cursor), Some(PAGE_SIZE));
    }
    Ok(animations)
}

/// One row of the table or the JSON array: an animation without its document.
#[derive(Serialize)]
struct AnimationRow {
    id: Uuid,
    title: String,
    #[serde(rename = "projectId")]
    project_id: Uuid,
    width: u16,
    height: u16,
    #[serde(rename = "frameCount")]
    frame_count: u16,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

impl AnimationRow {
    fn of(record: &AnimationRecord) -> Self {
        let updated_at = record.updated_at.format(&Rfc3339).unwrap_or_default();
        Self {
            id: record.id.uuid(),
            title: record.meta.title.as_str().to_owned(),
            project_id: record.project.uuid(),
            width: record.meta.width,
            height: record.meta.height,
            frame_count: record.meta.frame_count,
            updated_at,
        }
    }
}

fn print_json(animations: &[AnimationRecord]) {
    let rows: Vec<AnimationRow> = animations.iter().map(AnimationRow::of).collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&rows).unwrap_or_default()
    );
}

fn print_table(animations: &[AnimationRecord], messages: &Messages) {
    if animations.is_empty() {
        println!("{}", messages.text("cli.list.empty", &[]));
        return;
    }
    println!("{}", messages.text("cli.list.header", &[]));
    for animation in animations {
        let row = AnimationRow::of(animation);
        let size = format!("{}x{}", row.width, row.height);
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.id, row.title, row.project_id, size, row.frame_count, row.updated_at
        );
    }
}
