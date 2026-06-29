use clap::Parser;
use otctl::cli::{exit_code, run, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(output) => println!("{output}"),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(exit_code(&err));
        }
    }
}
