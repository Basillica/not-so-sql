use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: HashMap<usize, Row>,
    pub primary_key: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Row {
    pub data: HashMap<String, String>
}