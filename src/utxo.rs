// src/utxo.rs
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use sha256::digest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Utxo {
  pub input: Vec<String>, //old utxos to be spent
  pub output: String, //receiver address, design decision to only include one output
  pub amount: u64,
  pub hash: Option<String>
}

impl Utxo {
  pub fn new(input: Vec<String>, output: String, amount: u64) -> Self {
    Utxo { input: input.clone(), output: output.clone(), amount, hash: Some(Utxo::hash(input, output, amount)) }
  }

  fn hash (input: Vec<String>, output: String, amount: u64) -> String {
    let json = json_string(&(Utxo { input, output, amount, hash: None }));
    digest(json.unwrap())
  }
}