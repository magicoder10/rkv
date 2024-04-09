use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    OkValue { value: Option<String> },
    OkNoContent,
    ErrorKeyNotFound,
    ErrorUnknown{message: String}
}