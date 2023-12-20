use clap::{builder::ValueParser, Parser};
use sanctum_solana_cli_utils::{ConfigWrapper, TxSendMode};
use subcmd::Subcmd;

mod rpc_client;
mod subcmd;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = "slumlord solana program CLI")]
pub struct Args {
    #[clap(
        long,
        short,
        help = "path to solana CLI config",
        default_value = "",
        value_parser = ValueParser::new(ConfigWrapper::parse_from_path)
    )]
    pub config: ConfigWrapper,

    #[clap(
        long,
        short,
        help = "only simulate any transactions instead of sending them",
        default_value_t = false
    )]
    pub dry_run: bool,

    #[clap(subcommand)]
    pub subcmd: Subcmd,
}

impl Args {
    pub fn tx_send_mode(&self) -> TxSendMode {
        TxSendMode::from_should_dry_run(self.dry_run)
    }
}

fn main() {
    let args = Args::parse();
    args.subcmd.process(&args);
}
