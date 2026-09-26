//! Reading one's requests, and replying.

use time::Duration;

use super::{Harness, START, account};
use crate::paging::PageRequest;
use crate::support::ports::SupportMessage;
use crate::support::{Author, SupportError, SupportRequestId, SupportStatus};

#[tokio::test]
async fn a_person_lists_only_their_requests_from_the_latest() {
    let harness = Harness::new();
    let first = harness.send(account(1), "First").await;
    harness.clock.advance(Duration::minutes(1));
    let second = harness.send(account(1), "Second").await;
    harness.send(account(2), "Someone else's").await;

    let page = harness.support.list(account(1), PageRequest::default());

    let ids: Vec<_> = page.await.unwrap().items.iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![second.request.id, first.request.id]);
}

#[tokio::test]
async fn another_persons_request_is_not_found() {
    let harness = Harness::new();
    let sent = harness.send(account(1), "Mine").await;
    let id = sent.request.id;

    let read = harness.support.thread(account(2), id).await;
    let replied = harness.support.reply(account(2), (id, "Hi")).await;

    assert_eq!(read.unwrap_err(), SupportError::RequestNotFound);
    assert_eq!(replied.unwrap_err(), SupportError::RequestNotFound);
    let unknown = SupportRequestId::from_uuid(uuid::Uuid::from_u128(99));
    let missing = harness.support.thread(account(1), unknown).await;
    assert_eq!(missing.unwrap_err(), SupportError::RequestNotFound);
}

#[tokio::test]
async fn the_team_internal_notes_never_reach_the_person() {
    let harness = Harness::new();
    let sent = harness.send(account(1), "Help").await;
    let note = SupportMessage {
        id: uuid::Uuid::from_u128(77),
        author: Author::Team,
        body: "Internal: known issue".into(),
        created_at: START,
    };
    harness.requests.add_internal_note(sent.request.id, note);

    let thread = harness.support.thread(account(1), sent.request.id).await;

    let bodies: Vec<_> = thread
        .unwrap()
        .messages
        .into_iter()
        .map(|m| m.body)
        .collect();
    assert_eq!(bodies, vec!["Help".to_owned()]);
}

#[tokio::test]
async fn a_reply_moves_a_request_waiting_for_the_person_back_to_the_team() {
    let harness = Harness::new();
    let id = harness.send(account(1), "Help").await.request.id;
    harness
        .requests
        .set_status(id, SupportStatus::WaitingForUser);
    harness.clock.advance(Duration::hours(1));

    let reply = harness
        .support
        .reply(account(1), (id, " More details "))
        .await;

    let reply = reply.unwrap();
    assert_eq!(
        (reply.author, reply.body.as_str()),
        (Author::User, "More details")
    );
    let thread = harness.support.thread(account(1), id).await.unwrap();
    assert_eq!(thread.request.status, SupportStatus::InProgress);
    assert_eq!(thread.request.updated_at, START + Duration::hours(1));
    assert_eq!(thread.messages.len(), 2);
}

#[tokio::test]
async fn a_reply_keeps_any_other_open_status() {
    let harness = Harness::new();
    let id = harness.send(account(1), "Help").await.request.id;
    for status in [
        SupportStatus::New,
        SupportStatus::InProgress,
        SupportStatus::Resolved,
    ] {
        harness.requests.set_status(id, status);
        harness
            .support
            .reply(account(1), (id, "Ping"))
            .await
            .unwrap();
        let thread = harness.support.thread(account(1), id).await.unwrap();
        assert_eq!(thread.request.status, status);
    }
}

#[tokio::test]
async fn a_closed_request_refuses_replies() {
    let harness = Harness::new();
    let id = harness.send(account(1), "Help").await.request.id;
    harness.requests.set_status(id, SupportStatus::Closed);

    let refused = harness.support.reply(account(1), (id, "Hello?")).await;

    assert_eq!(refused.unwrap_err(), SupportError::RequestClosed);
    let thread = harness.support.thread(account(1), id).await.unwrap();
    assert_eq!(thread.messages.len(), 1);
}

#[tokio::test]
async fn an_empty_reply_is_refused_before_the_store_is_asked() {
    let harness = Harness::new();
    let id = harness.send(account(1), "Help").await.request.id;
    harness.requests.set_unavailable(true);

    let refused = harness.support.reply(account(1), (id, "  ")).await;

    assert_eq!(refused.unwrap_err(), SupportError::MessageLength);
}
