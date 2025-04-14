use std::env;

#[cfg(feature = "tokio")]
use tokio::sync::oneshot;

use crate::DehashedApi;
#[cfg(feature = "tokio")]
use crate::ScheduledRequest;
use crate::api::{Query, SearchType};

fn setup() -> (DehashedApi, String) {
    let api_key = env::var("API_KEY").unwrap();
    let search = env::var("SEARCH").unwrap();

    let api = DehashedApi::new(api_key).unwrap();

    (api, search)
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_scheduler() {
    let (api, search) = setup();

    let scheduler = api.start_scheduler();

    let sender = scheduler.retrieve_sender();

    let (tx, rx) = oneshot::channel();

    sender
        .send(ScheduledRequest::new(
            Query::Domain(SearchType::Simple(search)),
            tx,
        ))
        .await
        .unwrap();

    rx.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_normal() {
    let (api, search) = setup();

    api.search(Query::Domain(SearchType::Simple(search)))
        .await
        .unwrap();
}
