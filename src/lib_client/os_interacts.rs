use super::net_messages;
use lib_client::eventloop::EventLoop;
use md5;
use std::process;
use std::sync::mpsc::Sender;
use std::thread;
use sys_info;

pub fn get_hostname() -> Result<String, sys_info::Error> {
  let hostname = sys_info::hostname()?;
  Ok(hostname)
}

pub fn get_os_release() -> Result<String, sys_info::Error> {
  let os_release = sys_info::os_release()?;
  Ok(os_release)
}

pub fn get_hash() -> Result<String, sys_info::Error> {
  let input = format!("{}-{}", get_os_release()?, get_hostname()?);
  let hash = md5::compute(input);
  Ok(format!("{:x}", hash))
}

fn parse_command(command: String) -> Option<(String, Vec<String>)> {
  let splits: Vec<&str> = command.split(" ").collect();
  match splits.get(0) {
    Some(x) => {
      let main_command = x.to_string();
      if main_command.len() == 0 {
        return None;
      }
      let mut t = splits.clone();
      t.remove(0);
      let args = t.into_iter().map(|a| a.to_string()).collect();

      Some((main_command, args))
    }
    None => None,
  }
}

fn run_command(mut command: net_messages::Command) -> net_messages::Command {
  info!("CMD: |{}|", command.input.clone());
  let output = match parse_command(command.input.clone()) {
    Some((main_command, args)) => match process::Command::new(&main_command).args(&args).output() {
      Ok(output) => {
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        format!("status: {} \n {} {}", output.status, out, err)
      }
      Err(err) => format!("status: command failed! {}", err),
    },
    None => String::new(),
  };
  command.add_output(output);
  command
}

pub fn run_command_async(command: net_messages::Command, tx: Sender<EventLoop>) {
  thread::spawn(move || {
    let output = run_command(command);
    tx.send(EventLoop::CommandComplete(output))
      .unwrap_or_else(|err| error!("{}", err));
  });
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_parse_command() {
    use super::*;
    let parsed = parse_command("ls -a".to_string());
    assert_eq!(parsed, Some(("ls".to_string(), vec!["-a".to_string()])));
    let parsed = parse_command("pwd".to_string());
    assert_eq!(parsed, Some(("pwd".to_string(), Vec::new())));
  }
}
