//! Sending a request.

use super::{CONTEXT, Harness, START, account, submission};
use crate::support::ports::SupportMessage;
use crate::support::{
    Author, Category, SupportContext, SupportError, SupportStatus, SupportSubmission,
};

#[tokio::test]
async fn a_request_starts_new_with_its_message_first_and_its_context_stored() {
    let harness = Harness::new();

    let thread = harness.send(account(1), "  The canvas is blank.  ").await;

    let request = &thread.request;
    assert_eq!(request.category, Category::Bug);
    assert_eq!(request.status, SupportStatus::New);
    assert!(!request.has_screenshot);
    assert_eq!((request.created_at, request.updated_at), (START, START));
    let [SupportMessage { author, body, .. }] = thread.messages.as_slice() else {
        panic!("{thread:?}");
    };
    assert_eq!(
        (*author, body.as_str()),
        (Author::User, "The canvas is blank.")
    );
    let (context, screenshot_key) = harness.requests.stored(request.id).unwrap();
    assert_eq!(context, SupportContext::parse(CONTEXT.as_bytes()).unwrap());
    assert_eq!(context.screen, "/editor/:animationId");
    assert_eq!(screenshot_key, None);
}

#[tokio::test]
async fn a_request_records_its_category() {
    let harness = Harness::new();
    let data = SupportSubmission {
        category: "data_protection".into(),
        ..submission("Please erase my data.", None)
    };

    harness.support.create(account(1), data).await.unwrap();

    let events = harness.events.events();
    let [event] = events.as_slice() else {
        panic!("{events:?}")
    };
    assert_eq!(event.name, "support_request_created");
    assert_eq!(event.account, Some(account(1)));
    assert_eq!(
        event.properties,
        vec![("category", "data_protection".into())]
    );
}

#[tokio::test]
async fn an_unknown_category_is_refused() {
    let harness = Harness::new();
    let unknown = SupportSubmission {
        category: "praise".into(),
        ..submission("Great app.", None)
    };

    let refused = harness.support.create(account(1), unknown).await;

    assert_eq!(refused.unwrap_err(), SupportError::Category);
    assert!(harness.events.events().is_empty());
}

#[tokio::test]
async fn a_message_empty_or_too_long_is_refused_with_its_bounds() {
    let harness = Harness::new();
    let too_long = "a".repeat(life_pixel_core::limits::SUPPORT_MESSAGE_MAX_CHARS + 1);
    let longest = "é".repeat(life_pixel_core::limits::SUPPORT_MESSAGE_MAX_CHARS);

    for message in ["", "   \n", too_long.as_str()] {
        let refused = harness
            .support
            .create(account(1), submission(message, None))
            .await;
        let error = refused.unwrap_err();
        assert_eq!(error, SupportError::MessageLength);
        let params = crate::Coded::params(&error);
        assert_eq!(
            (params["min"].as_u64(), params["max"].as_u64()),
            (Some(1), Some(5_000))
        );
    }
    let accepted = harness
        .support
        .create(account(1), submission(&longest, None))
        .await;
    assert!(accepted.is_ok());
}

#[tokio::test]
async fn a_context_that_is_not_the_apps_is_malformed() {
    let harness = Harness::new();
    let with_ids = SupportSubmission {
        context: CONTEXT
            .replace(":animationId", "0190f1c2-7a4e-7b3c-9d2e-1f2a3b4c5d6e")
            .into_bytes(),
        ..submission("Hello", None)
    };

    let refused = harness.support.create(account(1), with_ids).await;

    assert_eq!(refused.unwrap_err(), SupportError::Malformed);
}
