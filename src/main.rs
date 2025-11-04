#[allow(unused_imports)]
use std::io::{self, Write};

const BUILD_IN_COMMANDS: [&str; 2] = ["exit", "echo"];
enum Command {
    CommandBuiltIn { builtin_command: BuiltInCommand },
    CommandNotFound { unknown_command_name: String },
}
enum BuiltInCommand {
    Exit { exit_code: i32 },
    Echo { content: String },
}

trait ParseInput {
    fn from_input(input: &str) -> Self;
}

impl ParseInput for Command {
    fn from_input(input: &str) -> Self {
        let input = input.trim();
        let args = input.split_whitespace().collect::<Vec<&str>>();
        // NOTE: builtin command
        for builtin in BUILD_IN_COMMANDS {
            if args[0] == builtin {
                match builtin {
                    "exit" => {
                        return Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Exit {
                                exit_code: if args.len() > 1 {
                                    args[1].parse().unwrap_or_default()
                                } else {
                                    0
                                },
                            },
                        }
                    }
                    "echo" => {
                        return Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Echo {
                                content: String::from(input.trim_start_matches("echo ")),
                            },
                        }
                    }
                    _ => {}
                }
            }
        }

        // NOTE: other command
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
            Command::CommandNotFound {
                unknown_command_name,
            } => {
                println!("{}: command not found", unknown_command_name);
            }
            Command::CommandBuiltIn { builtin_command } => match builtin_command {
                BuiltInCommand::Exit { exit_code: _ } => {
                    break;
                }
                BuiltInCommand::Echo { content } => {
                    println!("{}", content);
                }
            },
        }
    }
}
