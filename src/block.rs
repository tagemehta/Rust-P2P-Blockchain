// src/block.rs

use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub nonce: i64,
    pub hash: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,    
}

impl Block {
    pub fn to_json_string(&self) -> String {
        let str = json_string(&self);
        match str {
            Ok(v) => v,
            Err(e) => panic!("Error: {:?}", e),
        }
    }

    pub fn increment_nonce(&mut self) {
        self.header.nonce = &self.header.nonce + 1;
    }

    pub fn header_to_json_string(&self) -> String {
        let str = json_string(&self.header);
        match str {
            Ok(v) => v,
            Err(e) => panic!("Error: {:?}", e),
        }
    }
}