use sanctum_solana_test_utils::assert_custom_err;
use slumlord_interface::{borrow_ix, repay_ix, SlumlordError};
use slumlord_lib::{check_repaid_ix_full, BorrowFreeArgs, RepayFreeArgs};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::{
    common::{SlumlordBanksClient, SlumlordProgramTest},
    SLUMLORD_LAMPORTS,
};

mod good_program {
    use sanctum_misc_utils::load_accounts;
    use slumlord_interface::{
        borrow_invoke, borrow_ix, check_repaid_invoke, repay_invoke, BorrowAccounts,
        CheckRepaidAccounts, RepayAccounts, BORROW_IX_ACCOUNTS_LEN,
    };
    use slumlord_lib::BorrowFreeArgs;
    use solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    };

    sanctum_macros::declare_program_keys!("8kbLzKfKo5gjbGQf2HmULGGTXQx6hnfYGJ8inL1zvVeL", []);

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let borrow_accounts: BorrowAccounts = load_accounts(accounts)?;
        let system_program = &accounts[BORROW_IX_ACCOUNTS_LEN];
        // just borrows, then immediately repays + checkrepaid
        borrow_invoke(borrow_accounts)?;
        repay_invoke(RepayAccounts {
            slumlord: borrow_accounts.slumlord,
            src: borrow_accounts.dst,
            system_program,
        })?;
        check_repaid_invoke(CheckRepaidAccounts {
            slumlord: borrow_accounts.slumlord,
        })
    }

    pub fn good_ix(dst: Pubkey) -> Instruction {
        let mut ix = borrow_ix(BorrowFreeArgs { dst }).unwrap();
        ix.accounts.push(AccountMeta {
            pubkey: system_program::ID,
            is_signer: false,
            is_writable: false,
        });
        ix.accounts.push(AccountMeta {
            pubkey: slumlord_lib::program::ID,
            is_signer: false,
            is_writable: false,
        });
        ix.program_id = ID;
        ix
    }
}

trait GoodProgramTest {
    fn add_good_program(self) -> Self;
}

impl GoodProgramTest for ProgramTest {
    fn add_good_program(mut self) -> Self {
        // cant cargo-test-sbf since we dont actually build a
        // good_program.so
        // This line needs to come before add_program() to take effect
        self.prefer_bpf(false);
        self.add_program(
            "good_program",
            good_program::ID,
            processor!(good_program::process_instruction),
        );
        self
    }
}

#[tokio::test]
async fn good_program_fails_with_no_top_level_checkrepaid() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_good_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = good_program::good_ix(payer.pubkey());
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
async fn good_program_success() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_good_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = good_program::good_ix(payer.pubkey());
    let mut tx = Transaction::new_with_payer(
        &[ix, check_repaid_ix_full().unwrap()],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer], last_blockhash);
    banks_client.process_transaction(tx).await.unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn good_program_does_not_block_future_borrows() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS)
        .add_good_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let ix = good_program::good_ix(payer.pubkey());
    let mut tx = Transaction::new_with_payer(
        &[
            ix,
            borrow_ix(BorrowFreeArgs {
                dst: payer.pubkey(),
            })
            .unwrap(),
            repay_ix(RepayFreeArgs {
                src: payer.pubkey(),
            })
            .unwrap(),
            check_repaid_ix_full().unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer], last_blockhash);
    banks_client.process_transaction(tx).await.unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}
