use clap::Parser;

#[derive(Parser, Clone)]
#[command(version, about)]
pub struct Args {
    #[arg(help = "Serial port (as path to /dev/tty* or COM port)")]
    port: String,

    #[arg(help = "Baud rate", default_value = "9600")]
    baud: u32,
}

#[tokio::main]
async fn main() {
    let Args { port, baud } = Args::parse();

    ssi::run(&port, baud).await;
}
