// src/blockchain.rs
use crate::block::{Block, Header};
use crate::transaction::Transaction;
use crate::utxo::Utxo;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
// use crypto::digest::Digest;
// use crypto::sha2::Sha256;
// use crate::node::Node;
use sha256::digest;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub utxo_set: Vec<Utxo>, //utxos are a source of truth
    pub utxo_blocked: Vec<Utxo> //utxos as previously valid
}

impl Blockchain {
    pub fn new() -> Self {
        let coinbase_tx = Transaction::new(vec![], vec![(String::from("owner"), 1000)]);
        let coinbase_utxo = Utxo::new(String::from("owner"), 1000, coinbase_tx.clone().txid.unwrap(), 0);
        let genesis_block = Block {
            header: Header {
                index: 1,
                timestamp: 000,
                previous_hash: String::from("1"), // Dummy value for the first block
                nonce: 0,
                hash: Some(String::from("000")),
            },
            transactions: vec![coinbase_tx.clone()],
        };

        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: vec![],
            utxo_set: vec![coinbase_utxo.clone()],
            utxo_blocked: vec![coinbase_utxo],
        }
    }
    // Generate a proof given the previous block's header and the current block's content
    pub fn mine(&mut self) -> Block {
        let last_block = self.chain.last().unwrap();
        let mut to_add = Block {
            header: Header {
                index: last_block.header.index + 1,
                nonce: 0,
                previous_hash: last_block.header.hash.clone().unwrap(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                hash: None,
            },
            transactions: self.pending_transactions.clone(),
        };
        self.pending_transactions = vec![];
        let mut proof = Blockchain::gen_proof(&self, &to_add);

        while !Blockchain::is_valid_proof(&proof) {
            to_add.header.nonce += 1;
            proof = Blockchain::gen_proof(&self, &to_add);
        }

        to_add.header.hash = Some(proof);
        self.chain.push(to_add.clone());
        self.utxo_blocked = self.utxo_set.clone();
        to_add
    }

    pub fn gen_proof(&self, b: &Block) -> String {
        let s = format!(
            "{}",
            digest(
                self.chain
                    .get(self.chain.len() - 1)
                    .unwrap()
                    .header_to_json_string()
                    + b.to_json_string().as_str()
            )
        );

        s
    }

    pub fn is_valid_proof(hash: &String) -> bool {
        hash.to_owned().ends_with("00") //number of 0s determines difficulty
    }
   
    pub fn verify_transaction(&self, tx: &Transaction, utxos: &Vec<Utxo>) -> bool {
        let mut valid_funds = 0;
        let mut to_spend = 0;
        let _ = tx.output.iter().map(|x| to_spend+=x.1);
        //sum all utxos in transaction
        for utxo in utxos.iter() {
            if tx.input.contains(&utxo.hash.clone().unwrap()) {
                valid_funds += utxo.amount;
            }
        }

        valid_funds >= to_spend
    }
    
    //assumed that enough funds are present
    pub fn create_pending_tx(&mut self, tx: &Transaction) {
        
        let utxo_hashes = tx.input.clone();
        // let utxs = self.utxo_set.clone();
        // let last = utxs.into_iter().find(|x| x.hash.clone().unwrap().eq(&utxo_hashes.last().unwrap().clone()));
        self.utxo_set.retain(|x| {

          if utxo_hashes.contains(&x.clone().hash.unwrap()) {
            false
          }
          else {
            true
          }

        });
        for (i, (address, amount)) in tx.output.iter().enumerate() {
          self.utxo_set.push(Utxo::new( address.clone(), *amount, tx.clone().txid.unwrap(), i.try_into().unwrap()));
        }
        
        self.pending_transactions.push(tx.clone());
    }


    pub fn verify_block(&self, b: &Block) -> bool {
        let new_block = Block {
            header: Header {
                hash: None,
                ..b.header.clone()
            },
            transactions: b.transactions.clone(),
        };
        
        b.header
            .hash
            .clone()
            .unwrap()
            .eq(&self.gen_proof(&new_block))
            && Blockchain::is_valid_proof(&b.header.hash.clone().unwrap()) && b.header.index == self.chain.last().unwrap().header.index + 1
    }

    pub fn send_to(&mut self, from: String, to: String, amount: u64) -> Option<Transaction> {
        let mut utxos_to_spend: Vec<Utxo> = vec![];
        let mut valid_funds = 0;
        
        for utxo in self.utxo_set.iter() {
            if utxo.output.eq(&from) && valid_funds < amount {
                valid_funds += utxo.amount;
                utxos_to_spend.push(utxo.clone());
            }
        }
        match valid_funds >= amount {
            _ => {
              let utxo_hashes: Vec<String> = utxos_to_spend
              .iter()
              .map(|x| x.hash.clone().unwrap())
              .collect();
              let mut outputs = vec![(to.clone(), amount)];

              if valid_funds > amount { //PROBLEM
                outputs.push((from.clone(), valid_funds - amount));
              }
              let tx = Transaction::new(utxo_hashes, outputs);
              Blockchain::create_pending_tx(self, &tx);
              Some(tx)
            }
            // false => {
            //     None
            // },
        }
    }
    pub fn receive_transaction(&mut self, tx: &Transaction) {
        if Blockchain::verify_transaction(self, tx, &self.utxo_set) {
          Blockchain::create_pending_tx(self, tx);
        }
    }

    pub fn receive_block(&mut self, b: Block) {
      let mut valid = Blockchain::verify_block(self, &b);
      for tx in b.transactions.iter() {
        valid = valid && Blockchain::verify_transaction(self, tx, &self.utxo_blocked)
      }
      if valid {
        
        let mut to_process = vec![];
        //create all transactions that have not been pending on our chain. remove them from our transaction pool
        self.pending_transactions.retain(|t| {
          if !b.transactions.contains(t) {
            to_process.push(t.clone());
            true
          }
          else {
            false
          }
        });
        for tx in to_process {
          Blockchain::create_pending_tx(self, &tx)
        }
        self.chain.push(b);
      }
    }

    pub fn num_pending_tx(&self) -> usize {
      self.pending_transactions.len()
    }

    pub fn receive_chain(&mut self, b_chain: &Blockchain) -> bool {
      if b_chain.chain.len() >= self.chain.len() {
        let mut from_beginning = Blockchain::new();
        let mut chain_iter = b_chain.chain.iter();
        chain_iter.next(); //skip the genesis block
        let mut is_valid = true;
        'outer: for block in chain_iter {
          if from_beginning.verify_block(block) {
            for tx in block.transactions.iter() {
              if Blockchain::verify_transaction(self, tx, &from_beginning.utxo_set) {
                Blockchain::create_pending_tx(self, tx)
              }
              else {
                is_valid = false;
                break 'outer
              }
            }

            from_beginning.chain.push(block.clone());
          } else {
            is_valid = false;
            break;
          }
        }
        

        is_valid
      }
      else {
        false
      }
       // only change anything if received is a longer chain
    }


}

impl fmt::Display for Blockchain {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      let mut wallets:HashMap<String, u64> = HashMap::new();
      for utxo in self.utxo_set.iter() {
        wallets.entry(utxo.output.clone()).and_modify(|amt| *amt += utxo.amount).or_insert(utxo.amount);
      }

      let mut print_str = String::from("");
      for (addy, amt) in &wallets {
        print_str += &format!("{}: {}\n", addy, amt);
      }

      let mut wallets:HashMap<String, u64> = HashMap::new();
      print_str += "\nBlocked Wallets\n";
      for utxo in self.utxo_blocked.iter() {
        wallets.entry(utxo.output.clone()).and_modify(|amt| *amt += utxo.amount).or_insert(utxo.amount);
      }

      for (addy, amt) in &wallets {
        print_str += &format!("{}: {}\n", addy, amt);
      }
      write!(f, "Pending Wallets\n{}", print_str)
  }
}