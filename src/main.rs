use clap::Parser;

use video2document::cli::CliArgs;

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    match video2document::run_cli(args).await {
        Ok(report) => std::process::exit(report.code),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}
