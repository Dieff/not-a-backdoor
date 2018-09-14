use lib_client::eventloop::EventLoop;
use std::sync::mpsc::Sender;
use std::thread::{sleep, spawn};
use std::time::Duration;

fn timing_loop(duration: Duration, sender: Sender<EventLoop>) {
  loop {
    sleep(duration);
    sender.send(EventLoop::TimerOff).unwrap();
  }
}

pub fn make_timer(duration: Duration, sender: Sender<EventLoop>) {
  spawn(move || {
    timing_loop(duration, sender);
  });
}
