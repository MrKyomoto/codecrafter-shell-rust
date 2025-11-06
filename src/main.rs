#[allow(unused_imports)]
use std::io::{self, Write};
use std::{error::Error, fmt::Display, fs::metadata, path::PathBuf};

const BUILD_IN_COMMANDS: [&str; 3] = ["exit", "echo", "type"];
enum Command {
    CommandBuiltIn {
        builtin_command: BuiltInCommand,
    },
    CommandNotFound {
        unknown_command_name: String,
    },
    CommandExternal {
        command_name: String,
        args: Option<Vec<String>>,
    },
    Empty,
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

struct Input<'a> {
    raw: &'a str,
    trimmed: &'a str,
    args: Vec<&'a str>,
}

struct ParsedContext<'a> {
    input: Input<'a>,
    paths: &'a [String],
}

impl<'a> From<(&'a str, &'a [String])> for ParsedContext<'a> {
    fn from((input_str, paths): (&'a str, &'a [String])) -> Self {
        ParsedContext {
            input: Input::from(input_str),
            paths,
        }
    }
}

trait ParseInput {
    fn from_input(context: &ParsedContext) -> Result<Box<Self>, Box<dyn Error>>;
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(value: &'a str) -> Self {
        let trimed_input = value.trim();
        Input {
            raw: value,
            trimmed: trimed_input,
            args: trimed_input.split_whitespace().collect(),
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
            // paths.reverse();
        }
        None => {}
    }
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let context = ParsedContext::from((&input[..], &paths[..]));
        let command = Command::from_input(&context)?;
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
    fn from_input(context: &ParsedContext) -> Result<Box<Self>, Box<dyn Error>> {
        let input = &context.input;
        let paths = context.paths;
        if input.trimmed.is_empty() {
            return Ok(Box::new(Command::Empty));
        }
        // NOTE: builtin command
        for builtin in BUILD_IN_COMMANDS {
            if input.args[0] == builtin {
                let command_builtin = construct_builtin(input, paths, builtin)?;
                if let Some(command) = command_builtin {
                    return Ok(Box::from(command));
                }
            }
        }

        match input.trimmed {
            _ => {
                // NOTE: external command
                let command_external = construct_external_command(input, paths)?;
                if let Some(command) = command_external {
                    return Ok(Box::from(command));
                }

                let command = Command::CommandNotFound {
                    unknown_command_name: String::from(input.trimmed),
                };
                Ok(Box::from(command))
            }
        }
    }
}

fn construct_external_command(
    input: &Input<'_>,
    paths: &[String],
) -> Result<Option<Command>, Box<dyn Error>> {
    for path in paths {
        let executable_file_type = find_bin_from_path(path, &input.args[0])?;
        match executable_file_type {
            ExecutableFileType::Permission(_) => {
                let command = Command::CommandExternal {
                    command_name: input.args[0].to_string(),
                    args: {
                        if input.args.len() > 1 {
                            Some(
                                input.args[1..input.args.len()]
                                    .iter()
                                    .map(|&s| s.to_string())
                                    .collect(),
                            )
                        } else {
                            None
                        }
                    },
                };
                return Ok(Some(command));
            }
            ExecutableFileType::NoPermission => {}
            ExecutableFileType::NotFound => {}
        }
    }
    Ok(None)
}

fn construct_builtin(
    input: &Input<'_>,
    paths: &[String],
    builtin: &str,
) -> Result<Option<Command>, Box<dyn Error>> {
    match builtin {
        "exit" => {
            let command = Command::CommandBuiltIn {
                builtin_command: BuiltInCommand::Exit {
                    exit_code: if input.args.len() > 1 {
                        input.args[1].parse()?
                    } else {
                        0
                    },
                },
            };
            return Ok(Some(command));
        }
        "echo" => {
            let command = Command::CommandBuiltIn {
                builtin_command: BuiltInCommand::Echo {
                    content: String::from(input.trimmed.trim_start_matches("echo ")),
                },
            };
            return Ok(Some(command));
        }
        "type" => {
            if input.args.len() == 1 {
                // NOTE: 事实上这里应该返回的是提示用户type后边必须跟一条指令,
                // 不过我的enum目前没有这个内容,感觉加上去有点冗余,就先返回None
                return Ok(None);
                // break;
            };
            if BUILD_IN_COMMANDS.contains(&input.args[1]) {
                let command = Command::CommandBuiltIn {
                    builtin_command: BuiltInCommand::Type {
                        command_name: String::from(input.args[1]),
                        command_type: CommandType::BuiltIn,
                    },
                };
                return Ok(Some(command));
            }

            for path in paths {
                let executable_file_type = find_bin_from_path(path, &input.args[1])?;
                match executable_file_type {
                    ExecutableFileType::Permission(absolute_path) => {
                        let command = Command::CommandBuiltIn {
                            builtin_command: BuiltInCommand::Type {
                                command_name: String::from(input.args[1]),
                                command_type: CommandType::Other(absolute_path),
                            },
                        };
                        return Ok(Some(command));
                    }
                    ExecutableFileType::NoPermission => {}
                    ExecutableFileType::NotFound => {}
                }
            }

            let command = Command::CommandBuiltIn {
                builtin_command: BuiltInCommand::Type {
                    command_name: String::from(input.args[1]),
                    command_type: CommandType::Invalid,
                },
            };
            return Ok(Some(command));
        }
        // NOTE: 事实上这个函数绝对不会到这里
        _ => return Ok(None),
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
            Command::CommandExternal { command_name, args } => {
                let mut executer = std::process::Command::new(command_name);
                if let Some(args) = args {
                    executer.args(args);
                }
                let mut process = executer.spawn()?;
                let _status = process.wait()?;
            }
            Command::Empty => {}
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
