use async_trait::async_trait;
use sanctum_solana_test_utils::{ExtendedBanksClient, ExtendedProgramTest};
use slumlord_lib::program::SLUMLORD_ID;
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::account::Account;

pub trait SlumlordProgramTest {
    fn add_slumlord_program(self) -> Self;

    fn add_slumlord(self, lamports: u64) -> Self;
}

impl SlumlordProgramTest for ProgramTest {
    fn add_slumlord_program(mut self) -> Self {
        self.add_program(
            "slumlord",
            slumlord_lib::program::ID,
            processor!(slumlord::process_instruction),
        );
        self
    }

    fn add_slumlord(self, lamports: u64) -> Self {
        let account = Account {
            lamports,
            data: Vec::new(),
            owner: slumlord_lib::program::ID,
            executable: false,
            rent_epoch: u64::MAX,
        };
        self.add_account_chained(SLUMLORD_ID, account)
    }
}

#[async_trait]
pub trait SlumlordBanksClient {
    async fn get_slumlord_acc(&mut self) -> Account;

    async fn assert_slumlord_data_empty(&mut self);

    async fn assert_slumlord_balance(&mut self, expected_lamports: u64);
}

#[async_trait]
impl SlumlordBanksClient for BanksClient {
    async fn get_slumlord_acc(&mut self) -> Account {
        self.get_account_unwrapped(SLUMLORD_ID).await
    }

    async fn assert_slumlord_data_empty(&mut self) {
        let slumlord = self.get_slumlord_acc().await;
        assert!(slumlord.data.is_empty());
    }

    async fn assert_slumlord_balance(&mut self, expected_lamports: u64) {
        let slumlord = self.get_slumlord_acc().await;
        let actual_lamports = slumlord.lamports;
        assert_eq!(
            actual_lamports, expected_lamports,
            "expected {expected_lamports}, got {actual_lamports}",
        );
    }
}
