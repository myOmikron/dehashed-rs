use dehashed_rs::{DehashedApi, Query, SearchType};

// Tokio is not needed, but is used as example runtime
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let api_key = "<api_key>".to_string();

    // Create an api instance
    let api = DehashedApi::new(api_key).unwrap();

    // Query for the domain example.com
    match api
        .search(Query::Domain(SearchType::Simple("example.com".to_string())))
        .await
    {
        Ok(res) => {
            println!("{res:?}");
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
        }
    }

    // Query for example.com or example.org
    match api
        .search(Query::Domain(SearchType::Or(vec![
            SearchType::Simple("example.com".to_string()),
            SearchType::Simple("example.org".to_string()),
        ])))
        .await
    {
        Ok(res) => {
            println!("{res:?}");
        }
        Err(err) => {
            eprintln!("Error: {err:?}");
        }
    }
}
