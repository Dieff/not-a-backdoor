use lib_server::eventloop::EventLoop;
use std::io;
use std::sync::mpsc::Sender;

pub trait Listener {
  fn start_listening(&mut self, sender: Sender<EventLoop>) -> Result<(), io::Error>;
}
