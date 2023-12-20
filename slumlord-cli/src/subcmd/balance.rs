use clap::Args;
use solana_sdk::native_token::lamports_to_sol;

use crate::rpc_client::SlumlordRpcClient;

#[derive(Args, Debug)]
#[clap(long_about = "Read slumlord's current SOL balance")]
pub struct BalanceArgs;

impl BalanceArgs {
    pub fn process(&self, args: &crate::Args) {
        let client = args.config.rpc_client();

        let slumlord = client.get_slumlord_unwrapped();

        let lamports = slumlord.lamports;
        let sol = lamports_to_sol(lamports);

        let loan_lamports = lamports - 1;
        let loan_sol = lamports_to_sol(loan_lamports);

        println!("Total balance: {sol} SOL ({lamports} lamports)");
        println!("Loan amount: {loan_sol} SOL ({loan_lamports} lamports)");
    }
}
