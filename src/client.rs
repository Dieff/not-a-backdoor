extern crate md5;
extern crate rand;
extern crate serde_json;
extern crate sys_info;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate stderrlog;

mod lib_client;
mod net_messages;

use lib_client::senders::{udp, MessageCenter};
use lib_client::{
  eventloop::EventLoop,
  os_interacts::{get_hash, run_command_async},
  timer::make_timer,
};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

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
  make_timer(Duration::from_millis(1000), tx.clone());

  // Handles the selection and tracking of senders
  let mut msg_center = MessageCenter::new(tx.clone());

  let my_id = get_hash().unwrap();
  let sender = udp::UdpSender::new("[::1]:5353".parse().unwrap(), 0);
  msg_center.add_sender(sender);

  let mut cmd_buffer: Vec<net_messages::Command> = Vec::new();

  for msg in rx {
    match msg {
      EventLoop::CommandComplete(cmd) => {
        let output = cmd.output.clone();
        cmd_buffer.push(cmd);
        info!("cmd complete! {}", &output.unwrap());
      }
      EventLoop::MessageResponse(msg) => {
        for cmd in msg.commands {
          warn!("found a command");
          run_command_async(cmd, tx.clone());
        }
        info!("recieved a message response");
      }
      EventLoop::TimerOff => {
        info!("timer off");
        let commands = cmd_buffer.drain(..).collect();
        let msg = net_messages::ClientToServer::from_cmds(my_id.clone(), commands);
        msg_center.send_message(msg);
      }
      _ => {
        error!("error ocurred in top level event loop");
      }
    }
  }
}
