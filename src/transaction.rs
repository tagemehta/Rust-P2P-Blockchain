// src/transaction.rs
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use sha256::digest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub input: Vec<String>, // Vector of utxo hashes to consume
    pub output: Vec<(String, u64)>, // Vector (receiver address, amount)
    pub txid: Option<String> //
}

impl Transaction {
    pub fn new(input: Vec<String>, output: Vec<(String, u64)>) -> Self {
      Transaction { input: input.clone(), output: output.clone(), txid: Some(Transaction::hash(input, output)) }
    }

    fn hash (input: Vec<String>, output: Vec<(String, u64)>) -> String {
        let json = json_string(&(Transaction { input, output, txid: None }));
        digest(json.unwrap())
    }
}
