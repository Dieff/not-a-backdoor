use super::net_messages::{ClientToServer, ServerToClient};
use lib_server::{
  eventloop::{ClientMessage, EventLoop},
  listener::Listener,
};
use mio::*;
use serde_json;
use std::{io, string::FromUtf8Error, sync::mpsc::Sender, thread::spawn, time::Duration};

pub struct UdpListener {
  port: u32,
}

impl UdpListener {
  pub fn new(port: u32) -> UdpListener {
    UdpListener { port }
  }
}

impl Listener for UdpListener {
  fn start_listening(&mut self, sender: Sender<EventLoop>) -> Result<(), io::Error> {
    let bind_addr = format!("[::]:{}", self.port);
    let socket = net::UdpSocket::bind(&bind_addr.parse().unwrap())?;

    spawn(move || {
      let poll = Poll::new().unwrap();
      poll
        .register(&socket, Token(0), Ready::readable(), PollOpt::edge())
        .unwrap_or_else(|err| error!("{}", err));

      let mut events = Events::with_capacity(5000);
      let mut data_buffer: [u8; 2048] = [0; 2048];

      loop {
        poll
          .poll(&mut events, Some(Duration::from_millis(200)))
          .unwrap_or_default();
        for _ in events.iter() {
          println!("new event");
          match socket.recv_from(&mut data_buffer) {
            Ok((data_len, address)) => {
              // to avoid a string full of Nul ut8 chars
              // we only copy the length of the message from the buffer into the string
              let trimmed = data_buffer[0..data_len].to_vec();
              println!("recieved {} bytes from {}", data_len, address);

              match decode_incoming(trimmed) {
                Ok(client_msg) => {
                  // need to clone this so we can give it to the closure
                  let socket_clone = socket.try_clone().unwrap();

                  // send a result back to the main event loop
                  let result = ClientMessage {
                    commands: client_msg.commands,
                    client_id: client_msg.id,
                    source: address,
                    reply: Box::new(move |srv_reply: ServerToClient| {
                      match encode_outgoing(srv_reply) {
                        Ok(bytes) => {
                          socket_clone
                            .send_to(&bytes, &address)
                            .unwrap_or_else(|err| {
                              error!("sending data {}", err);
                              1
                            });
                        }
                        Err(err) => {
                          error!("encoding - {}", err);
                        }
                      }
                    }),
                  };

                  sender
                    .send(EventLoop::NewClientMessage(result))
                    .unwrap_or_else(|err| error!("{}", err));
                }
                Err(err) => {
                  error!("error in encoding");
                  sender
                    .send(EventLoop::DecodeError)
                    .unwrap_or_else(|err| error!("{}", err));
                }
              }
            }
            Err(err) => error!("recieving {}", err),
          };
          // overwrite the buffer with 0s again
          data_buffer = [0; 2048];
        }
      }
    });
    Ok(())
  }
}

enum DecodeError {
  Bytes(FromUtf8Error),
  Json(serde_json::Error),
}

fn decode_incoming(data_in: Vec<u8>) -> Result<ClientToServer, DecodeError> {
  match String::from_utf8(data_in) {
    Ok(msg) => {
      println!("recieved message {}", msg);
      match ClientToServer::from_str(msg) {
        Ok(client_msg) => Ok(client_msg),
        Err(json_error) => Err(DecodeError::Json(json_error)),
      }
    }
    Err(oops) => Err(DecodeError::Bytes(oops)),
  }
}

fn encode_outgoing(msg: ServerToClient) -> Result<Vec<u8>, serde_json::Error> {
  let msg_string = serde_json::to_string(&msg)?;
  Ok(msg_string.as_bytes().to_vec())
}
