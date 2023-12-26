use sanctum_solana_test_utils::assert_custom_err;
use slumlord_interface::{borrow_ix, SlumlordError};
use slumlord_lib::{check_repaid_ix_full, BorrowFreeArgs};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::{
    common::{SlumlordBanksClient, SlumlordProgramTest},
    SLUMLORD_LAMPORTS,
};

mod evil_err_catcher_program {
    use sanctum_misc_utils::load_accounts;
    use slumlord_interface::{
        borrow_invoke, borrow_ix, check_repaid_invoke, BorrowAccounts, CheckRepaidAccounts,
    };
    use slumlord_lib::BorrowFreeArgs;
    use solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    sanctum_macros::declare_program_keys!("8kbLzKfKo5gjbGQf2HmULGGTXQx6hnfYGJ8inL1zvVeL", []);

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let accounts: BorrowAccounts = load_accounts(accounts)?;
        borrow_invoke(accounts)?;
        // CheckRepaid without repaying, but catch the error
        // and do nothing so that the transaction proceeds
        let _err = check_repaid_invoke(CheckRepaidAccounts {
            slumlord: accounts.slumlord,
        });
        Ok(())
    }

    pub fn evil_err_catcher_ix(dst: Pubkey) -> Instruction {
        let mut ix = borrow_ix(BorrowFreeArgs { dst }).unwrap();
        ix.accounts.push(AccountMeta {
            pubkey: slumlord_lib::program::ID,
            is_signer: false,
            is_writable: false,
        });
        ix.program_id = ID;
        ix
    }
}

trait EvilErrCatcherProgramTest {
    fn add_evil_err_catcher_program(self) -> Self;
}

impl EvilErrCatcherProgramTest for ProgramTest {
    fn add_evil_err_catcher_program(mut self) -> Self {
        // cant cargo-test-sbf since we dont actually build a
        // evil_err_catcher.so
        // This line needs to come before add_program() to take effect
        self.prefer_bpf(false);
        self.add_program(
            "evil_err_catcher",
            evil_err_catcher_program::ID,
            processor!(evil_err_catcher_program::process_instruction),
        );
        self
    }
}

#[tokio::test]
async fn evil_err_catcher_fails_with_no_top_level_checkrepaid() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_evil_err_catcher_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = evil_err_catcher_program::evil_err_catcher_ix(payer.pubkey());
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], last_blockhash);
    let err = banks_client.process_transaction(tx).await.unwrap_err();
    assert_custom_err(err, SlumlordError::NoSucceedingCheckRepaid);
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn evil_err_catcher_fails() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_evil_err_catcher_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = evil_err_catcher_program::evil_err_catcher_ix(payer.pubkey());
    let mut tx = Transaction::new_with_payer(
        &[ix, check_repaid_ix_full().unwrap()],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer], last_blockhash);
    let err = banks_client.process_transaction(tx).await.unwrap_err();
    assert_custom_err(err, SlumlordError::InsufficientRepay);
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn evil_err_catcher_blocks_future_borrows() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_evil_err_catcher_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = evil_err_catcher_program::evil_err_catcher_ix(payer.pubkey());
    let mut tx = Transaction::new_with_payer(
        &[
            ix,
            borrow_ix(BorrowFreeArgs {
                dst: payer.pubkey(),
            })
            .unwrap(),
            check_repaid_ix_full().unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer], last_blockhash);
    let err = banks_client.process_transaction(tx).await.unwrap_err();
    assert_custom_err(err, SlumlordError::BorrowAlreadyActive);
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}
