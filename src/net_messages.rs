use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
  pub input: String,
  pub output: Option<String>,
  pub id: usize,
}

impl Command {
  pub fn from_input(input: String, id: usize) -> Command {
    Command {
      output: None,
      input,
      id,
    }
  }

  pub fn add_output(&mut self, output: String) {
    self.output = Some(output);
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientToServer {
  pub commands: Vec<Command>,
  pub id: String,
}

impl ClientToServer {
  pub fn from_str(string: String) -> Result<ClientToServer, serde_json::Error> {
    serde_json::from_str(&string)
  }
  pub fn from_cmds(id: String, commands: Vec<Command>) -> ClientToServer {
    ClientToServer { id, commands }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerToClient {
  pub commands: Vec<Command>,
}

impl ServerToClient {
  pub fn from_str(string: String) -> Result<ServerToClient, serde_json::Error> {
    serde_json::from_str(&string)
  }
  pub fn from_cmds(commands: Vec<Command>) -> ServerToClient {
    ServerToClient { commands }
  }
}
