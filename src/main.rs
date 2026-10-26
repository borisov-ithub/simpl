use std::env as std_env;
use std::fs;

fn main() {
    let args: Vec<String> = std_env::args().collect();

    let mut check_only = false;
    let mut file: Option<String> = None;
    let mut program_args: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-h" | "--help" if file.is_none() => {
                print!("{}", simpl::messages::cli_help());
                return;
            }
            "-v" | "--version" if file.is_none() => {
                println!("{}", simpl::messages::cli_version(env!("CARGO_PKG_VERSION")));
                return;
            }
            "-r" | "--repl" if file.is_none() => {
                if let Err(e) = simpl::repl::run() {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
                return;
            }
            "-c" | "--check" if file.is_none()=> {
                check_only = true;
            }
            other if other.starts_with('-') && file.is_none() => {
                eprintln!("{}", simpl::messages::cli_unknown_flag(other));
                std::process::exit(1);
            }
            other => {
                if file.is_some() {
                    // всё после файла — аргументы программы
                    program_args.extend(args[i..].iter().cloned());
                    break;
                }
                file = Some(other.to_string());
            }
        }
        i += 1;
    }

    let path = match file {
        Some(p) => p,
        None => {
            eprintln!("{}", simpl::messages::cli_missing_file());
            std::process::exit(1);
        }
    };

    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", simpl::messages::cli_read_error(&path, &e));
            std::process::exit(1);
        }
    };

    if check_only {
        match simpl::check_source(&src) {
            Ok(()) => {
                println!("{}", simpl::messages::cli_check_ok(&path));
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let path_buf = std::path::PathBuf::from(path.as_str());
    if let Err(e) = simpl::run_source_with_path(&src, Some(path_buf), program_args) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}