use super::net_messages;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, SystemTime};

struct CommandResult {
  input: String,
  output: Option<String>,
  sent: i64,
  recieved: i64,
  id: usize,
}

impl CommandResult {
  pub fn new(input: String, id: usize) -> CommandResult {
    CommandResult {
      output: None,
      sent: 0,
      recieved: 0,
      input,
      id,
    }
  }

  pub fn set_output(&mut self, output: String) {
    self.output = Some(output);
  }
}

pub struct Client {
  commands: HashMap<usize, CommandResult>,
  ip: IpAddr,
  os: String,
  last_checkup: SystemTime,
}

impl Client {
  fn new(ip: IpAddr, os: String) -> Client {
    Client {
      commands: HashMap::new(),
      last_checkup: SystemTime::now(),
      ip,
      os,
    }
  }

  fn touch(&mut self) {
    self.last_checkup = SystemTime::now()
  }

  fn get_elapsed(&self) -> Duration {
    self.last_checkup.elapsed().unwrap()
  }

  fn commands_complete(&mut self, commands: Vec<net_messages::Command>) {
    for cmd in commands {
      match (self.commands.get_mut(&cmd.id), cmd.output) {
        (Some(a), Some(b)) => a.set_output(b),
        _ => (),
      };
    }
  }

  fn unfinished_cmds(&self) -> Vec<net_messages::Command> {
    self
      .commands
      .values()
      .filter(|c| c.output.is_none())
      .map(|c| net_messages::Command::from_input(c.input.clone(), c.id))
      .collect()
  }

  fn add_command(&mut self, input: String) {
    let id = self.commands.len();
    self.commands.insert(id, CommandResult::new(input, id));
  }
}

pub struct ClientTracker {
  clients: HashMap<String, Client>,
}

impl ClientTracker {
  pub fn new() -> ClientTracker {
    ClientTracker {
      clients: HashMap::new(),
    }
  }

  pub fn add_client(&mut self, id: String, ip: IpAddr, os: String) {
    self.clients.insert(id, Client::new(ip, os));
    println!("added client");
  }

  pub fn check_client(&mut self, id: &str) {
    match self.clients.get_mut(id) {
      Some(client) => client.touch(),
      _ => (),
    }
  }

  pub fn has_client(&self, id: &str) -> bool {
    self.clients.contains_key(id)
  }

  // records commands that were completed by the client
  pub fn finished_commands(&mut self, id: &str, cmds: Vec<net_messages::Command>) {
    match self.clients.get_mut(id) {
      Some(client) => client.commands_complete(cmds),
      _ => (),
    };
  }

  // returns commands that have yet to be sent to the client
  pub fn unfinished_cmds(&self, id: &str) -> Option<Vec<net_messages::Command>> {
    match self.clients.get(id) {
      Some(client) => Some(client.unfinished_cmds()),
      None => None,
    }
  }

  pub fn new_command(&mut self, input: String) {
    for client in self.clients.values_mut() {
      client.add_command(input.clone());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::net::Ipv4Addr;

  #[test]
  fn test_has_client() {
    let mut tracker = ClientTracker::new();
    assert!(!tracker.has_client("asdf"));
    tracker.add_client(
      "asdfasdf".to_owned(),
      IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
      "Linux".to_owned(),
    );
    assert!(tracker.has_client("asdfasdf"));
  }

  #[test]
  fn test_adding_cmds() {
    let mut tracker = ClientTracker::new();
    tracker.add_client(
      "asdfasdf".to_owned(),
      IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
      "Linux".to_owned(),
    );
    tracker.new_command("testing".to_owned());
    let res = tracker.unfinished_cmds("asdfasdf");
    assert!(res.is_some());
    assert_eq!(res.unwrap().len(), 1);
    assert!(tracker.unfinished_cmds("asd").is_none());
  }
}
