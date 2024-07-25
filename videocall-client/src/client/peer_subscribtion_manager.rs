use super::{
    media_management_interface::{send_command, ManagementCommand},
    send_packet::SendPacket,
};
use crate::crypto::aes::Aes128State;
use log::debug;
use std::{collections::HashSet, rc::Rc};

const LIMIT: usize = 20;

pub enum Reason {
    AutoSubscribe(String),
}

#[derive(Debug)]
pub struct PeerSubscribtionManager {
    userid: String,
    aes: Rc<Aes128State>,
    followed_peers: HashSet<String>,
}

impl PeerSubscribtionManager {
    pub fn new(userid: String, aes: Rc<Aes128State>) -> Self {
        Self {
            userid,
            aes,
            followed_peers: HashSet::new(),
        }
    }

    pub fn subscribe(&mut self, reason: Reason, packet_sender: &impl SendPacket) {
        let peer = match reason {
            Reason::AutoSubscribe(peer) => {
                if self.followed_peers.len() >= LIMIT {
                    Err(format!(
                        "Could not add more peers, current={} limit={}",
                        self.followed_peers.len(),
                        LIMIT
                    ))
                } else if self.followed_peers.insert(peer.clone()) {
                    Ok(peer)
                } else {
                    Err(format!("Could not add peer={}, already subscribed", peer))
                }
            }
        };

        match peer {
            Ok(peer) => send_command(
                packet_sender,
                &self.aes,
                self.userid.clone(),
                ManagementCommand::SubscribeTo(peer),
            ),
            Err(msg) => debug!("{}", msg),
        }
    }

    pub fn unsubscribe(&mut self, peer: &String) {
        self.followed_peers.remove(peer);
    }

    pub fn is_subscribed_to(&self, peer: &String) -> bool {
        self.followed_peers.contains(peer)
    }
}
