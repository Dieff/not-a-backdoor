use super::super::eventloop::EventLoop;
use super::net_messages;
use lib_client::senders::sender_trait::MessageSender;
use serde_json;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::mpsc::{SendError, Sender};
use std::thread::{sleep, spawn};
use std::time;

pub struct UdpSender {
  dest: SocketAddr,
  id: u32,
}

impl UdpSender {
  pub fn new(dest: SocketAddr, id: u32) -> UdpSender {
    UdpSender { dest, id }
  }
}

impl MessageSender for UdpSender {
  fn send_message(&mut self, msg: net_messages::ClientToServer, sender: Sender<EventLoop>) {
    match (serde_json::to_string(&msg), UdpSocket::bind("[::]:0")) {
      (Ok(msg_str), Ok(socket)) => {
        let timeout = time::Duration::from_secs(1);
        // let's set some timeouts on the socket so we don't have
        // to do this manually
        socket
          .set_write_timeout(Some(timeout.clone()))
          .unwrap_or_else(|err| error!("{}", err));
        socket
          .set_read_timeout(Some(timeout.clone()))
          .unwrap_or_else(|err| error!("{}", err));

        match socket.connect(&self.dest) {
          Ok(_) => {
            spawn(move || match socket.send(&msg_str.into_bytes()) {
              Ok(_) => {
                wait_for_response(socket, sender).unwrap();
              }
              Err(err) => {
                error!("{}", err);
                sender.send(EventLoop::MessageSendFailure).unwrap();
              }
            });
          }
          Err(err) => {
            error!("{}", err);
            sender
              .send(EventLoop::MessageSendFailure)
              .unwrap_or_default();
            return;
          }
        }
        // disconnect?
      }
      _ => println!("failed to encode message"),
    };
  }

  fn get_id(&self) -> u32 {
    self.id
  }
}

fn wait_for_response(
  socket: UdpSocket,
  sender: Sender<EventLoop>,
) -> Result<(), SendError<EventLoop>> {
  let mut buffer = [0; 2048];

  let size = match socket.recv(&mut buffer) {
    Ok(data_len) => data_len,
    Err(err) => {
      error!("Recv: {}", err);
      0
    }
  };

  if size == 0 {
    sender.send(EventLoop::MessageSendFailure)?;
    return Ok(());
  }

  match String::from_utf8(buffer[0..size].to_vec()) {
    Ok(msg_str) => match net_messages::ServerToClient::from_str(msg_str) {
      Ok(msg) => {
        sender.send(EventLoop::MessageResponse(msg))?;
      }
      Err(err) => {
        error!("{}", err);
        sender.send(EventLoop::MessageSendFailure)?;
      }
    },
    Err(err) => {
      error!("{}", err);
      sender.send(EventLoop::MessageSendFailure)?;
    }
  }
  Ok(())
}
