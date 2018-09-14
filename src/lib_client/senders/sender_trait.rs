use super::super::eventloop::EventLoop;
use super::net_messages;
use std::sync::mpsc::Sender;

pub trait MessageSender {
  fn send_message(&mut self, msg: net_messages::ClientToServer, tx: Sender<EventLoop>);
  fn get_id(&self) -> u32;
}
