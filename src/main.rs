use pathsearch::find_executable_in_path;
use std::{
    io::{self, Write},
    os::unix::process::CommandExt,
    process::{self, Command},
};
const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let input = command.trim();

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "cd" => {
                if args.is_empty() {
                    eprintln!("cd: missing operand");
                    continue;
                }

                let mut dir = args[0].to_string();

                // ~

                if dir.starts_with("~") {
                    if let Ok(home) = std::env::var("HOME") {
                        if dir == "~" {
                            dir = home;
                        } else if dir.starts_with("~/") {
                            dir = format!("{}/{}", home, &dir[2..]);
                        } else {
                            eprintln!("cd: HOME not set");
                            continue;
                        }
                    }
                }

                match std::env::set_current_dir(&dir) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("cd: {}: No such file or directory", dir);
                    }
                }
            }

            "exit" => process::exit(0),
            "echo" => {
                println!("{}", args.join(" "));
            }
            " " => continue,
            "pwd" => {
                println!("{}", std::env::current_dir().unwrap().display());
            }

            "type" => {
                if args.is_empty() {
                    println!("type: missing operand");
                    continue;
                }

                let target = args[0];

                if BUILTINS.contains(&target) {
                    println!("{} is a shell builtin", target);
                } else if let Some(path) = find_executable_in_path(args[0]) {
                    println!("{} is {}", args[0], path.display());
                } else {
                    println!("{}: not found", target);
                }
            }
            _ => {
                use std::path::PathBuf;

                let path = if cmd.contains("/") {
                    Some(PathBuf::from(cmd))
                } else {
                    find_executable_in_path(cmd)
                };

                if let Some(path) = path {
                    let status = Command::new(path).arg0(cmd).args(args).status();
                    match status {
                        Ok(status) => {
                            if !status.success() {
                                eprintln!("process exited with {}", status);
                            }
                        }
                        Err(e) => {
                            eprintln!("failed to execute {}: {}", cmd, e);
                        }
                    }
                } else {
                    println!("{}: command not found", cmd);
                }
            }
        }
    }
}

