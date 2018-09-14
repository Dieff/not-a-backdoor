use super::super::net_messages;
use std::fmt::Display;
use std::net::SocketAddr;

pub struct ClientMessage {
  pub commands: Vec<net_messages::Command>,
  pub source: SocketAddr,
  pub client_id: String,
  pub reply: Box<Fn(net_messages::ServerToClient) + Send>,
}

pub enum EventLoop {
  NewClientMessage(ClientMessage),
  NewCommand(String),
  DecodeError,
}
