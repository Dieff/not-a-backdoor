use lib_server::eventloop;
use std::io;
use std::sync::mpsc;
use std::thread;

pub fn stdin_reader(tx: mpsc::Sender<eventloop::EventLoop>) {
  thread::spawn(move || {
    loop {
      // initialize an empty buffer after every stdin message
      let mut buf = String::new();
      match io::stdin().read_line(&mut buf) {
        Ok(_) => {
          tx.send(eventloop::EventLoop::NewCommand(buf)).unwrap();
        }
        Err(e) => println!("could not read from stdin {}", e),
      };
    }
  });
}
