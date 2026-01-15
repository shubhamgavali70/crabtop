use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "port-usage")]
#[command(about = "Shows CPU & memory usage of process running on a port")]
pub struct Cli {
    #[arg(short, long)]
    pub port: u16,

    #[arg(long, help = "Watch CPU & memory usage live")]
    pub watch: bool,
}
