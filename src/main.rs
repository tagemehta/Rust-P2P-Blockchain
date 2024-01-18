pub mod blockchain;
pub mod node;
pub mod transaction;
pub mod block;
pub mod utxo;

use crate::node::{Node, MyBehaviour, MyBehaviourEvent};
use futures::stream::StreamExt;
use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::{io, io::AsyncBufReadExt, select};
use tracing_subscriber::EnvFilter;





#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  
    // chain.gen_proof(Block {
    //     header: Header {
    //         previous_hash: String::from(""),
    //         index: 0,
    //         timestamp: 0,
    //         proof: 0,
    //         nonce: 0,
    //     },
    //     transactions: vec![],
    // });
  // chain.send_to(String::from("owner"), String::from("rec"), 1000);
  // print!("{:?}", chain.mine());
  let _ = tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .try_init();

  let mut swarm = libp2p::SwarmBuilder::with_new_identity()
      .with_tokio()
      .with_tcp(
          tcp::Config::default(),
          noise::Config::new,
          yamux::Config::default,
      )?
      .with_quic()
      .with_behaviour(|key| {
          // To content-address message, we can take the hash of message and use it as an ID.
          let message_id_fn = |message: &gossipsub::Message| {
              let mut s = DefaultHasher::new();
              message.data.hash(&mut s);
              gossipsub::MessageId::from(s.finish().to_string())
          };

          // Set a custom gossipsub configuration
          let gossipsub_config = gossipsub::ConfigBuilder::default()
              .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
              .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
              .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
              .build()
              .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

          // build a gossipsub network behaviour
          let gossipsub = gossipsub::Behaviour::new(
              gossipsub::MessageAuthenticity::Signed(key.clone()),
              gossipsub_config,
          )?;

          let mdns =
              mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
          Ok(MyBehaviour { gossipsub, mdns })
      })?
      .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
      .build();

  // Create a Gossipsub topic
  let topic_tx = gossipsub::IdentTopic::new("transactions");
  // subscribes to our topic
  swarm.behaviour_mut().gossipsub.subscribe(&topic_tx)?;

  // Read full lines from stdin
  let mut stdin = io::BufReader::new(io::stdin()).lines();

  // Listen on all interfaces and whatever port the OS assigns
  swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
  swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
  let mut chain_node = Node::new();
  println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");
  // Kick it off
  loop {
      select! {
          Ok(Some(line)) = stdin.next_line() => {

            chain_node.rec_input(line, &topic_tx, &mut swarm);
            
        
          },
          event = swarm.select_next_some() => match event {
              SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                  for (peer_id, _multiaddr) in list {
                      println!("mDNS discovered a new peer: {peer_id}");
                      swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

                  }
              },
              SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                  for (peer_id, _multiaddr) in list {
                      println!("mDNS discover peer has expired: {peer_id}");
                      swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                  }
              },
              SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                  propagation_source: peer_id,
                  message_id: id,
                  message,
              })) => {
                let msg = String::from_utf8_lossy(&message.data);
                chain_node.receive_message(msg.to_string());
                
                println!(
                      "Got message: '{}' with id: {id} from peer: {peer_id}",
                      String::from_utf8_lossy(&message.data),
                  )
                },
              SwarmEvent::NewListenAddr { address, .. } => {
                  println!("Local node is listening on {address}");
              }
              _ => {}
          },

          b = async {chain_node.num_pending_tx() >= 2} => {
            if b {
              chain_node.mine(&topic_tx, &mut swarm);
            }
          }
      }
  }
}