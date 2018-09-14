use super::super::net_messages;
use super::eventloop::EventLoop;
use rand::{thread_rng, Rng, ThreadRng};
use std::sync::mpsc::Sender;

pub mod sender_trait;
pub mod udp;

pub struct MessageCenter<T: sender_trait::MessageSender> {
  senders: Vec<T>,
  tx: Sender<EventLoop>,
  rand: ThreadRng,
}

impl<T: sender_trait::MessageSender> MessageCenter<T> {
  pub fn new(tx: Sender<EventLoop>) -> MessageCenter<T> {
    MessageCenter {
      senders: Vec::new(),
      rand: thread_rng(),
      tx,
    }
  }

  pub fn add_sender(&mut self, sender: T) {
    self.senders.push(sender);
  }

  pub fn send_message(&mut self, msg: net_messages::ClientToServer) {
    match self.rand.choose_mut(&mut self.senders) {
      Some(sender) => {
        sender.send_message(msg, self.tx.clone());
      }
      None => (),
    }
  }
}
