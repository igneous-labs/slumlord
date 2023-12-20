use clap::Subcommand;

use self::{balance::BalanceArgs, init::InitArgs};

mod balance;
mod init;

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    Init(InitArgs),
    Balance(BalanceArgs),
}

impl Subcmd {
    pub fn process(&self, args: &crate::Args) {
        match self {
            Self::Init(a) => a.process(args),
            Self::Balance(a) => a.process(args),
        }
    }
}
