#[allow(unused_imports)]
use std::io::{self, Write};

const BUILD_IN_COMMANDS: [&str; 1] = ["exit"];
enum Command {
    ExitCommand { exit_code: i32 },
    CommandNotFound { unknown_command_name: String },
}

trait ParseInput {
    fn from_input(input: &str) -> Self;
}

impl ParseInput for Command {
    fn from_input(input: &str) -> Self {
        let input = input.trim();
        if input.starts_with("exit ") {
            let args = input.split_whitespace().collect::<Vec<&str>>();
            return Command::ExitCommand {
                exit_code: args[1].parse().unwrap(),
            };
        }
        match input {
            _ => {
                return Command::CommandNotFound {
                    unknown_command_name: String::from(input),
                }
            }
        }
    }
}
fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = Command::from_input(&input);
        match command {
            Command::ExitCommand { exit_code } => {
                break;
            }
            Command::CommandNotFound {
                unknown_command_name,
            } => {
                println!("{}: command not found", unknown_command_name);
            }
        }
    }
}
