use libp2p::Swarm;

use serde_json::{from_str, to_string, Error};
use regex::Regex;

use crate::blockchain::Blockchain;
use libp2p::gossipsub::{IdentTopic, Behaviour};
use libp2p::swarm::NetworkBehaviour;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: Behaviour,
    pub mdns: crate::mdns::tokio::Behaviour,
}
pub struct Node {
  pub chain: Blockchain,
}

impl Node{
  pub fn new() -> Self {
    Node {chain: Blockchain::new()}

  }
  pub fn receive_message (&mut self, msg: String) {
    if msg.starts_with("Transaction: ") {
      let tx_rec = from_str(&msg.replace("Transaction: ", ""));
      match tx_rec {
        Err(e) => println!("{:?}", e),
        Ok(tx) => {self.chain.create_pending_tx(&tx)}
      };
      
      println!("{}", &self.chain);
    }
    else if msg.starts_with("Block: ") {
      let block_rec = from_str(&msg.replace("Block: ", ""));
      match block_rec {
        Err(e) => println!("{:?}", e),
        Ok(b) => self.chain.receive_block(b)
      }
      println!("{}", &self.chain);
    }
    else if msg.starts_with("Blockchain: ") {
      let chain_rec: Result<Blockchain, Error> = from_str(&msg.replace("Blockchain: ", ""));
      match chain_rec {
        Err(e) => println!("{:?}", e),
        Ok(b) => {
          if self.chain.receive_chain(&b) {
            self.chain = b;
            println!("chain accepted");
          }
          else {
            println!("Chain rejected");
          }
        }
      }
    }
  }

  pub fn rec_input(&mut self, msg: String, topic_tx: &IdentTopic, swarm: &mut Swarm<MyBehaviour>,) {
    if msg.eq("start") {
      if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_tx.clone(), ("Blockchain: ".to_string() + &to_string(&self.chain).unwrap() ).as_bytes()) {
        println!("Publish error: {e:?}");  
      }
    }
    else {
      match parse_send_input(&msg) {
        Some((amt, send, rec)) => {
          match self.chain.send_to(send, rec, amt) {
            Some(tx) => {
              if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_tx.clone(), ("Transaction: ".to_string() + &to_string(&tx).unwrap() ).as_bytes()) {
                println!("Publish error: {e:?}");  
              }
            },
            None => println!("Invalid transaction")
            
          }
          
      },
        None => {
          println!("Invalid input format");
        }
    }
  }
  }
  pub fn mine(&mut self, topic_tx: &IdentTopic, swarm: &mut Swarm<MyBehaviour>,) {
    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic_tx.clone(), ("Block: ".to_string() + &to_string(&self.chain.mine()).unwrap() ).as_bytes()) {
      println!("Publish error: {e:?}");  
    }
  }

  pub fn num_pending_tx(&self) -> usize {
    self.chain.num_pending_tx()
  }

  pub fn send_chain(&self) -> String {
    to_string(&self.chain).unwrap() //unwrapping because it's a valid struct
  }

  // pub fn reconcile_chain(&mut self, msg: String) {
  //   if msg.starts_with("Blockchain: ") {
  //     let tx_rec = from_str(&msg.replace("Transaction: ", ""));
  //     match tx_rec {
  //       Err(e) => println!("{:?}", e),
  //       Ok(tx) => {self.chain.create_pending_tx(&tx)}
  //     };
      
  //     println!("{}", &self.chain);
  //   }
  // }
}

fn parse_send_input(input_string: &str) -> Option<(u64, String, String)> {
  // Define a regular expression pattern to match the input format
  let pattern = r"send (\d+) to (\w+) from (\w+)";
  let regex = Regex::new(pattern).expect("Invalid regex pattern");

  // Use regex to extract information from the input
  if let Some(captures) = regex.captures(input_string) {
      // Extracting values from captured groups
      let amount: u64 = captures[1].parse().expect("Failed to parse amount");
      let recipient_address = captures.get(2).map_or("", |m| m.as_str()).to_string();
      let sender_address = captures.get(3).map_or("", |m| m.as_str()).to_string();

      Some((amount, sender_address, recipient_address))
  } else {
      None // Return None if the input doesn't match the expected format
  }


}