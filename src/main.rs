#[allow(unused_imports)]
use std::io::{self, Write};
use std::{error::Error, fmt::Display, fs::metadata, path::PathBuf};

const BUILD_IN_COMMANDS: [&str; 3] = ["exit", "echo", "type"];
enum Command {
    CommandBuiltIn { builtin_command: BuiltInCommand },
    CommandNotFound { unknown_command_name: String },
}
enum BuiltInCommand {
    #[allow(dead_code)]
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut paths: Vec<String> = Vec::new();
    match std::env::var_os("PATH") {
        Some(val) => {
            paths = std::env::split_paths(&val)
                .map(|path| path.to_str().unwrap().to_string())
                .collect();
            // paths.reverse();
        }
        None => {}
    }
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = Command::from_input(&input, &paths)?;
        let is_continue = command.handle()?;
        if !is_continue {
            break;
        }
    }
    Ok(())
}

impl ParseInput for Command {
    // TODO: 这个函数还是太长了,
    // 由于构造command需要用到很多传入的参数所以...目前能想到的解决办法只有重构from_input的参数,
    // 大概是写一个结构体, 可能包含原始input,以及分割后的args还有环境变量一类的
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
                            let executable_file_type = find_bin_from_path(path, &args[1])?;
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
fn find_bin_from_path(path: &str, bin: &str) -> Result<ExecutableFileType, Box<dyn Error>> {
    let bin_path = PathBuf::from(path).join(bin);
    if !bin_path.exists() {
        return Ok(ExecutableFileType::NotFound);
    }

    let meta = metadata(&bin_path)?;
    if !meta.is_file() {
        return Ok(ExecutableFileType::NotFound);
    }

    #[cfg(unix)]
impl Command {}
    {
        use std::os::unix::fs::PermissionsExt;

        // NOTE: permission mask bit 0o111
        if (meta.permissions().mode() & 0o111) != 0 {
            return Ok(ExecutableFileType::Permission(
                (&bin_path).to_string_lossy().to_string(),
            ));
        } else {
            return Ok(ExecutableFileType::NoPermission);
        }
    }
}

impl Command {
    fn handle(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            Command::CommandNotFound {
                unknown_command_name,
            } => {
                println!("{}: command not found", unknown_command_name);
            }
            Command::CommandBuiltIn { builtin_command } => {
                return builtin_command.handle();
            }
        }
        Ok(true)
    }
}

impl BuiltInCommand {
    fn handle(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            BuiltInCommand::Exit { exit_code: _ } => {
                return Ok(false);
            }
            BuiltInCommand::Echo { content } => {
                println!("{}", content);
            }
            BuiltInCommand::Type {
                command_name,
                command_type,
            } => {
                println!("{}{}", command_name, command_type.to_string());
            }
        }
        Ok(true)
    }
}
impl Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::BuiltIn => {
                write!(f, " is a shell builtin")
            }
            CommandType::Other(path) => {
                write!(f, " is {}", path)
            }
            CommandType::Invalid => {
                write!(f, ": not found")
            }
        }
    }
}
