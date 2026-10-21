use std::env as std_env;
use std::fs;

fn main() {
    let args: Vec<String> = std_env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", simpl::messages::cli_usage());
        std::process::exit(1);
    }
    let path = &args[1];
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", simpl::messages::cli_read_error(path, &e));
            std::process::exit(1);
        }
    };
    if let Err(e) = simpl::run_source(&src) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}