use super::net_messages;

// struct FromServer

pub enum EventLoop {
  CommandComplete(net_messages::Command),
  MessageResponse(net_messages::ServerToClient),
  MessageSendFailure,
  TimerOff,
}
