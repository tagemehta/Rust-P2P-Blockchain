// src/utxo.rs
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use sha256::digest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Utxo {
  pub output: String, //receiver address, design decision to only include one output
  pub amount: u64,
  pub hash: Option<String>,
  pub txid: String, //hash of the transaction being spent
  pub index: u32 // index in the transaction array
}

impl Utxo {
  pub fn new(output: String, amount: u64, txid: String, index: u32) -> Self {
    Utxo { output: output.clone(), amount, txid: txid.clone(), hash: Some(Utxo::hash(output, amount, txid, index)), index }
  }

  fn hash (output: String, amount: u64, txid: String, index: u32) -> String {
    let json = json_string(&(Utxo { output, amount, hash: None, txid, index }));
    digest(json.unwrap())
  }
}