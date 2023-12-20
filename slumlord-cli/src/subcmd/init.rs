use clap::Args;
use sanctum_solana_cli_utils::TxSendingRpcClient;
use sanctum_system_program_lib::{transfer_ix, TransferKeys};
use slumlord_lib::{init_ix_full, program::SLUMLORD_ID};
use solana_sdk::{message::Message, native_token::sol_to_lamports, transaction::Transaction};

use crate::rpc_client::SlumlordRpcClient;

#[derive(Args, Debug)]
#[clap(long_about = "Initialize the slumlord PDA with the given SOL amount")]
pub struct InitArgs {
    #[clap(
        help = "Amount in SOL to initialize the slumlord PDA with. Must be >= rent_exempt_min(0)"
    )]
    pub init_sol: f64,

    #[clap(
        long,
        short,
        help = "if true, run transaction even if slumlord already initialized, effectively sending SOL to it. If false, do nothing if slumlord already initialized."
    )]
    pub force: bool,
}

impl InitArgs {
    pub fn process(&self, args: &crate::Args) {
        let payer = args.config.signer();
        let client = args.config.rpc_client();

        if client.get_slumlord().is_some() {
            println!("slumlord already initialized");
            if !self.force {
                return;
            }
        }

        let msg = Message::new(
            &[
                transfer_ix(
                    TransferKeys {
                        from: payer.pubkey(),
                        to: SLUMLORD_ID,
                    },
                    sol_to_lamports(self.init_sol),
                ),
                init_ix_full().unwrap(),
            ],
            Some(&payer.pubkey()),
        );
        let blockhash = client.get_latest_blockhash().unwrap();
        let tx = Transaction::new(&[payer.as_ref()], msg, blockhash);
        client.send_or_sim_tx(&tx, args.tx_send_mode());
    }
}
