use clap::Parser;
use murmur::cli::{commands, Cli};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => {
            if let Err(e) = commands::dispatch(cmd) {
                eprintln!("Error: {:#}", e);
                std::process::exit(1);
            }
        }
        None => {
            use clap::CommandFactory;
            let _ = Cli::command().print_help();
            println!();
        }
    }
}
