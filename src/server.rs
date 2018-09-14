extern crate mio;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate stderrlog;

mod lib_server;
mod net_messages;

use lib_server::{
  client_tracker::ClientTracker, eventloop::EventLoop, listener::Listener, stdin, udp::UdpListener,
};
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() {
  // init logging if the feature is set
  #[cfg(feature = "log_to_stderr")]
  {
    stderrlog::new()
      .module(module_path!())
      .verbosity(10)
      .init()
      .unwrap();
    warn!("logging is turned on");
  }

  let (tx, rx): (Sender<EventLoop>, Receiver<EventLoop>) = channel();

  let mut clients = ClientTracker::new();

  let mut udp53 = UdpListener::new(5353);
  udp53.start_listening(tx.clone()).unwrap();

  stdin::stdin_reader(tx.clone());

  for msg in rx {
    match msg {
      EventLoop::NewClientMessage(client_msg) => {
        info!("new client message coming in {}", client_msg.source);
        if !clients.has_client(&client_msg.client_id) {
          clients.add_client(
            client_msg.client_id.clone(),
            client_msg.source.ip(),
            "test".to_owned(),
          );
        }
        for item in &client_msg.commands {
          match &item.output {
            Some(ref ss) => println!("{}", ss),
            &None => (),
          };
        }
        clients.check_client(&client_msg.client_id);
        clients.finished_commands(&client_msg.client_id, client_msg.commands);
        let cmds = match clients.unfinished_cmds(&client_msg.client_id) {
          Some(new_cmds) => new_cmds,
          None => Vec::new(),
        };

        // Unbox and call the reply function
        (client_msg.reply)(net_messages::ServerToClient::from_cmds(cmds));
      }
      EventLoop::NewCommand(mut cmd) => {
        // The last character of the string will be a newline
        // got to parse the string
        cmd.pop();
        if cmd.len() > 1 {
          clients.new_command(cmd);
        }
      }
      EventLoop::DecodeError => {
        error!("an error ocurred when decoding an incoming message");
      }
    }
  }
}
