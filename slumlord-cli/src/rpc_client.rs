use slumlord_lib::program::SLUMLORD_ID;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;

pub trait SlumlordRpcClient {
    fn get_slumlord(&self) -> Option<Account>;

    fn get_slumlord_unwrapped(&self) -> Account {
        let opt = self.get_slumlord();
        match opt {
            Some(s) => s,
            None => panic!("slumlord account does not exist"),
        }
    }
}

impl SlumlordRpcClient for RpcClient {
    fn get_slumlord(&self) -> Option<Account> {
        self.get_account_with_commitment(&SLUMLORD_ID, self.commitment())
            .unwrap()
            .value
    }
}
