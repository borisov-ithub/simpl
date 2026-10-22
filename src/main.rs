use std::env as std_env;
use std::fs;

fn main() {
    let args: Vec<String> = std_env::args().collect();

    let mut check_only = false;
    let mut file: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "-h" | "--help" => {
                print!("{}", simpl::messages::cli_help());
                return;
            }
            "-v" | "--version" => {
                println!("{}", simpl::messages::cli_version(env!("CARGO_PKG_VERSION")));
                return;
            }
            "-r" | "--repl" => {
                eprintln!("{}", simpl::messages::cli_repl_not_implemented());
                std::process::exit(2);
            }
            "-c" | "--check" => {
                check_only = true;
            }
            other if other.starts_with('-') => {
                eprintln!("{}", simpl::messages::cli_unknown_flag(other));
                std::process::exit(1);
            }
            other => {
                if file.is_some() {
                    eprintln!("{}", simpl::messages::cli_too_many_files());
                    std::process::exit(1);
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

    let path_buf = std::path::PathBuf::from(&path);

    if let Err(e) = simpl::run_source_with_path(&src, Some(path_buf)) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}