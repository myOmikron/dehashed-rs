use std::fmt::{Display, Formatter, Write};
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(feature = "tokio")]
use tokio::time::sleep;
use tracing::{debug, error, warn};

#[cfg(feature = "tokio")]
use crate::Scheduler;
use crate::error::DehashedError;
use crate::res::{Entry, Response};

const URL: &str = "https://api.dehashed.com/v2/search";
const RESERVED: [char; 21] = [
    '+', '-', '=', '&', '|', '>', '<', '!', '(', ')', '{', '}', '[', ']', '^', '"', '~', '*', '?',
    ':', '\\',
];

fn escape(q: &str) -> String {
    let mut s = String::new();
    for c in q.chars() {
        if RESERVED.contains(&c) {
            s.write_str(&format!("\\{c}")).unwrap();
        } else {
            s.write_char(c).unwrap();
        }
    }
    s
}

/// A specific search type
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum SearchType {
    /// Search for a simple pattern
    Simple(String),
    /// Search for an exact pattern
    Exact(String),
    /// A regex search pattern
    Regex(String),
    /// Add multiple [SearchType]s with an OR
    Or(Vec<SearchType>),
    /// Add multiple [SearchType]s with an AND
    And(Vec<SearchType>),
}

impl Display for SearchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SearchType::Simple(x) => x.clone(),
                SearchType::Exact(x) => format!("\"{}\"", escape(x)),
                SearchType::Regex(x) => format!("/{}/", escape(x)),
                SearchType::Or(x) => x
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" OR "),
                SearchType::And(x) => x
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            }
        )
    }
}

/// A query for dehashed
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum Query {
    /// Search for an email
    Email(SearchType),
    /// Search for an ip address
    IpAddress(SearchType),
    /// Search for an username
    Username(SearchType),
    /// Search for an password
    Password(SearchType),
    /// Search for an hashed password
    HashedPassword(SearchType),
    /// Search for a name
    Name(SearchType),
    /// Search for a domain
    Domain(SearchType),
    /// Search for a vin
    Vin(SearchType),
    /// Search for a phone
    Phone(SearchType),
    /// Search for an address
    Address(SearchType),
}

impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Query::Email(x) => format!("email:{x}"),
                Query::IpAddress(x) => format!("ip_address:{x}"),
                Query::Username(x) => format!("username:{x}"),
                Query::Password(x) => format!("password:{x}"),
                Query::HashedPassword(x) => format!("hashed_password:{x}"),
                Query::Name(x) => format!("name:{x}"),
                Query::Domain(x) => format!("domain:{x}"),
                Query::Vin(x) => format!("vin:{x}"),
                Query::Phone(x) => format!("phone:{x}"),
                Query::Address(x) => format!("address:{x}"),
            }
        )
    }
}

/// The result of a search query
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SearchResult {
    /// A list of results
    pub entries: Vec<Entry>,
    /// The remaining balance
    pub balance: usize,
}

/// The instance of the dehashed api
#[derive(Clone, Debug)]
pub struct DehashedApi {
    client: Client,
}

impl DehashedApi {
    /// Create a new instance of the SDK.
    ///
    /// **Parameter**:
    /// - `email`: The mail address that is used for authentication
    /// - `api_key`: The api key for your account (found on your profile page)
    ///
    /// This method fails if the [Client] could not be constructed
    pub fn new(api_key: String) -> Result<Self, DehashedError> {
        let mut header_map = HeaderMap::new();
        header_map.insert("Accept", HeaderValue::from_static("application/json"));
        header_map.insert("Dehashed-Api-Key", HeaderValue::from_str(&api_key)?);

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .https_only(true)
            .default_headers(header_map)
            .build()?;

        Ok(Self { client })
    }

    async fn raw_req(
        &self,
        size: usize,
        page: usize,
        query: String,
    ) -> Result<Response, DehashedError> {
        let res = self
            .client
            .post(URL)
            .json(&json!({"query": query, "size": size, "page": page}))
            .send()
            .await?;

        let status = res.status();
        let raw = res.text().await?;
        debug!("status code: {status}. Raw: {raw}");
        if status == StatusCode::from_u16(302).unwrap() {
            Err(DehashedError::InvalidQuery)
        } else if status == StatusCode::from_u16(400).unwrap() {
            Err(DehashedError::Unknown(raw))
        } else if status == StatusCode::from_u16(401).unwrap() {
            Err(DehashedError::Unauthorized)
        } else if status == StatusCode::from_u16(200).unwrap() {
            match serde_json::from_str(&raw) {
                Ok(result) => Ok(result),
                Err(err) => {
                    error!("Error deserializing data: {err}. Raw data: {raw}");
                    Err(DehashedError::Unknown(raw))
                }
            }
        } else {
            warn!("Invalid response, status code: {status}. Raw: {raw}");

            Err(DehashedError::Unknown(raw))
        }
    }

    /// Query the API
    ///
    /// Please note, that dehashed has a ratelimit protection active, that bans every account
    /// that is doing more than 5 req / s.
    ///
    /// This method will take care of pagination and will delay requests if necessary.
    pub async fn search(&self, query: Query) -> Result<SearchResult, DehashedError> {
        let q = query.to_string();
        debug!("Query: {q}");

        let mut search_result = SearchResult {
            entries: vec![],
            balance: 0,
        };
        for page in 1.. {
            let res = self.raw_req(10_000, page, q.clone()).await?;

            if let Some(entries) = res.entries {
                for entry in entries {
                    search_result.entries.push(entry)
                }
            }

            search_result.balance = res.balance;

            if res.total < page * 10_000 {
                break;
            }

            #[cfg(feature = "tokio")]
            sleep(Duration::from_millis(200)).await;
        }

        Ok(search_result)
    }

    /// Start a new scheduler.
    ///
    /// The [Scheduler] manages stay in bounds of the rate limit of the unhashed API.
    /// It lets you push queries and receive the results.
    #[cfg(feature = "tokio")]
    pub fn start_scheduler(&self) -> Scheduler {
        Scheduler::new(self)
    }
}
