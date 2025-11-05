#[allow(unused_imports)]
use std::io::{self, Write};
use std::{error::Error, ops::Deref};

const BUILD_IN_COMMANDS: [&str; 3] = ["exit", "echo", "type"];
enum Command {
    CommandBuiltIn { builtin_command: BuiltInCommand },
    CommandNotFound { unknown_command_name: String },
}
enum BuiltInCommand {
    Exit {
        exit_code: i32,
    },
    Echo {
        content: String,
    },
    Type {
        command_name: String,
        command_type: CommandType,
    },
}
enum CommandType {
    BuiltIn,
    Other(String),
    Invalid,
}

enum ExecutableFileType {
    Permission(String),
    NoPermission,
    NotFound,
}

trait ParseInput {
    fn from_input(input: &str, paths: &Vec<String>) -> Result<Box<Self>, Box<dyn Error>>;
}

impl ParseInput for Command {
    fn from_input(input: &str, paths: &Vec<String>) -> Result<Box<Self>, Box<dyn Error>> {
        let input = input.trim();
        let args = input.split_whitespace().collect::<Vec<&str>>();
        // NOTE: builtin command
        for builtin in BUILD_IN_COMMANDS {
            if args[0] == builtin {
                match builtin {
                    "exit" => {
                        let command = Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Exit {
                                exit_code: if args.len() > 1 { args[1].parse()? } else { 0 },
                            },
                        };
                        return Ok(Box::from(command));
                    }
                    "echo" => {
                        let command = Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Echo {
                                content: String::from(input.trim_start_matches("echo ")),
                            },
                        };
                        return Ok(Box::from(command));
                    }
                    "type" => {
                        if args.len() == 1 {
                            break;
                        };
                        if BUILD_IN_COMMANDS.contains(&args[1]) {
                            let command = Command::CommandBuiltIn {
                                builtin_command: BuiltInCommand::Type {
                                    command_name: String::from(args[1]),
                                    command_type: CommandType::BuiltIn,
                                },
                            };
                            return Ok(Box::from(command));
                        }

                        for path in paths {
                            let executable_file_type = read_bin_from_path(path, &args[1])?;
                            match executable_file_type {
                                ExecutableFileType::Permission(absolute_path) => {
                                    let command = Command::CommandBuiltIn {
                                        builtin_command: BuiltInCommand::Type {
                                            command_name: String::from(args[1]),
                                            command_type: CommandType::Other(absolute_path),
                                        },
                                    };
                                    return Ok(Box::from(command));
                                }
                                ExecutableFileType::NoPermission => {}
                                ExecutableFileType::NotFound => {}
                            }
                        }

                        let command = Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Type {
                                command_name: String::from(args[1]),
                                command_type: CommandType::Invalid,
                            },
                        };
                        return Ok(Box::from(command));
                    }
                    _ => {}
                }
            }
        }

        // NOTE: other command
        match input {
            _ => {
                let command = Command::CommandNotFound {
                    unknown_command_name: String::from(input),
                };
                Ok(Box::from(command))
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths: Vec<String> = Vec::new();
    match std::env::var_os("PATH") {
        Some(val) => {
            paths = std::env::split_paths(&val)
                .map(|path| path.to_str().unwrap().to_string())
                .collect();
        }
        None => {}
    }
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = Command::from_input(&input, &paths)?;
        match command.deref() {
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
                BuiltInCommand::Type {
                    command_name,
                    command_type,
                } => match command_type {
                    CommandType::BuiltIn => {
                        println!("{} is a shell builtin", command_name)
                    }
                    CommandType::Other(path) => {
                        println!("{} is {}", command_name, path)
                    }
                    CommandType::Invalid => {
                        println!("{}: not found", command_name)
                    }
                },
            },
        }
    }
    Ok(())
}

fn read_bin_from_path(path: &str, bin: &str) -> Result<ExecutableFileType, Box<dyn Error>> {
    let entries = std::fs::read_dir(path)?;

    for entry in entries {
        let dir_entry = entry?;

        let meta = dir_entry.metadata()?;
        if meta.is_file() {
            if dir_entry.file_name() == bin {
                if !meta.permissions().readonly() {
                    return Ok(ExecutableFileType::Permission(String::from(
                        dir_entry.path().to_string_lossy().to_string(),
                    )));
                } else {
                    return Ok(ExecutableFileType::NoPermission);
                }
            }
        }
    }

    Ok(ExecutableFileType::NotFound)
}
