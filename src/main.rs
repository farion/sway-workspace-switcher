use std::env;
use swayipc::{Connection, Fallible, Node, Output};

pub struct Switcher {
    direction: String,
    connection: Connection,
    cur_workspace_num: i32,
    outputs: Vec<Output>,
    cur_output_name: String,
}

impl Switcher {
    pub fn new(direction: &String) -> Self {
        let mut connection = Connection::new().unwrap();
        let cur_workspace = connection.get_outputs().unwrap()
            .into_iter()
            .filter(|o| o.focused == true)
            .nth(0)
            .unwrap()
            .current_workspace
            .unwrap();
        let cur_workspace_num = cur_workspace.split_whitespace().next().unwrap().parse().unwrap();
        let mut outputs: Vec<Output> = connection.get_outputs().unwrap()
            .into_iter()
            .filter(|o| o.active == true)
            .collect();
        outputs.sort_by(|a, b| a.rect.x.partial_cmp(&b.rect.x).unwrap());
        let cur_output_name = connection.get_tree().unwrap()
            .find(|n| n.name == Some(String::from("root")))
            .unwrap()
            .nodes
            .into_iter()
            .flat_map(|n| n.nodes)
            .filter(|n| n.num == Some(cur_workspace_num))
            .nth(0)
            .unwrap()
            .output
            .unwrap();
        return Switcher {
            direction: direction.clone(),
            connection,
            cur_workspace_num,
            outputs,
            cur_output_name,
        };
    }

    fn get_output_index(&self, output_name: String) -> usize {
        return self.outputs
            .iter()
            .position(|o| o.name == output_name).unwrap();
    }

    fn get_first_workspace_on_output(&mut self, output: Output) -> i32 {
        return self.connection.get_workspaces().unwrap()
            .into_iter()
            .filter(|w| w.output == output.name)
            .nth(0)
            .unwrap()
            .num;
    }

    fn get_last_workspace_on_output(&mut self, output: Output) -> i32 {
        return self.connection.get_workspaces().unwrap()
            .into_iter()
            .filter(|w| w.output == output.name)
            .last()
            .unwrap()
            .num;
    }

    fn move_to_workspace(&mut self, workspace:i32) {
        if let Err(e) = self.connection.run_command(format!("workspace number {}", workspace)) {
            println!("Error switching workspace: {}", e.to_string());
        }
    }

    fn go_to_workspace(&mut self, mut to_workspace: i32) {

        if self.direction == "prev" {
            to_workspace = to_workspace - 1;
        } else if self.direction == "next" {
            to_workspace = to_workspace + 1;
        } else {
            println!("Invalid parameter. Use prev or next only");
        }

        let output_index = self.get_output_index(self.cur_output_name.clone());
        if to_workspace > 10 {
            let cand_index = output_index + 1;
            if self.outputs.len() > cand_index {
                let workspace = self.get_first_workspace_on_output(self.outputs[cand_index].clone());
                self.move_to_workspace(workspace.clone());
            }
            return;
        } else if to_workspace < 1 {
            let cand_index = output_index - 1;
            if self.outputs.len() > cand_index {
                let workspace = self.get_last_workspace_on_output(self.outputs[cand_index].clone());
                self.move_to_workspace(workspace.clone());
            }
            return;
        }

        let workspaces:Vec<Node> = self.connection.get_tree().unwrap()
            .find(|n| n.name == Some(String::from("root")))
            .unwrap()
            .nodes
            .into_iter()
            .flat_map(|n| n.nodes)
            .filter(|n| n.num == Some(to_workspace))
            .collect();

        let mut to_output:String = String::from("None");

        if workspaces.len() > 0 {
            to_output = workspaces[0].clone().output.unwrap();
        }

        if to_output == self.cur_output_name {
            self.move_to_workspace(to_workspace.clone());

        } else {
            self.go_to_workspace(to_workspace.clone());
        }
    }

    pub fn start(&mut self) {
        self.go_to_workspace(self.cur_workspace_num.clone());
    }
}

fn main() -> Fallible<()> {
    let args: Vec<String> = env::args().collect();
    Switcher::new(&args[1]).start();
    Ok(())
}
