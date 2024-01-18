// src/transaction.rs
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use sha256::digest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub input: Vec<String>, //input utxo.hash
    pub output: String, //output utxo.hash
    pub amount: u64,
    pub hash: Option<String>
}

impl Transaction {
    pub fn new(input: Vec<String>, output: String, amount: u64) -> Self {
      Transaction { input: input.clone(), output: output.clone(), amount, hash: Some(Transaction::hash(input, output, amount)) }
    }

    fn hash (input: Vec<String>, output: String, amount: u64) -> String {
        let json = json_string(&(Transaction { input, output, amount, hash: None }));
        digest(json.unwrap())
    }
}
