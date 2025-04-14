use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub email: Option<Vec<String>>,
    pub ip_address: Option<Vec<String>>,
    pub username: Option<Vec<String>>,
    pub password: Option<Vec<String>>,
    pub hashed_password: Option<Vec<String>>,
    pub name: Option<Vec<String>>,
    pub dob: Option<Vec<String>>,
    pub license_plate: Option<Vec<String>>,
    pub address: Option<Vec<String>>,
    pub phone: Option<Vec<String>>,
    pub company: Option<Vec<String>>,
    pub url: Option<Vec<String>>,
    pub social: Option<Vec<String>>,
    pub cryptocurrency_address: Option<Vec<String>>,
    pub database_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub balance: usize,
    pub entries: Option<Vec<Entry>>,
    pub took: String,
    pub total: usize,
}
