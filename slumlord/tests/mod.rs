mod common;
mod cpi;

use common::*;

use sanctum_solana_test_utils::{assert_custom_err, ExtendedBanksClient};
use sanctum_system_program_lib::{transfer_ix, TransferKeys};
use slumlord_interface::{borrow_ix, repay_ix, SlumlordError};
use slumlord_lib::{
    check_repaid_ix_full, init_ix_full, program::SLUMLORD_ID, BorrowFreeArgs, RepayFreeArgs,
};
use solana_program::hash::Hash;
use solana_program_test::ProgramTest;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

// 0.1 SOL
const SLUMLORD_LAMPORTS: u64 = 100_000_000;

fn borrow_donate_check_repaid_tx(payer: &Keypair, last_blockhash: Hash) -> Transaction {
    let borrow_ix = borrow_ix(BorrowFreeArgs {
        dst: payer.pubkey(),
    })
    .unwrap();
    let donate_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        SLUMLORD_LAMPORTS - 1,
    );
    let check_repaid_ix = check_repaid_ix_full().unwrap();
    let mut tx = Transaction::new_with_payer(
        &[borrow_ix, donate_ix, check_repaid_ix],
        Some(&payer.pubkey()),
    );
    tx.sign(&[payer], last_blockhash);
    tx
}

fn fund_and_init_tx(payer: &Keypair, last_blockhash: Hash) -> Transaction {
    let fund_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        SLUMLORD_LAMPORTS,
    );
    let mut tx =
        Transaction::new_with_payer(&[fund_ix, init_ix_full().unwrap()], Some(&payer.pubkey()));
    tx.sign(&[payer], last_blockhash);
    tx
}

#[tokio::test]
async fn init() {
    let pt = ProgramTest::default().add_slumlord_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    banks_client
        .process_transaction(fund_and_init_tx(&payer, last_blockhash))
        .await
        .unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;

    // check functionality
    let tx = borrow_donate_check_repaid_tx(&payer, last_blockhash);
    banks_client.process_transaction(tx).await.unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn init_twice_ok() {
    let pt = ProgramTest::default().add_slumlord_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    banks_client
        .process_transaction(fund_and_init_tx(&payer, last_blockhash))
        .await
        .unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;

    // init again
    let mut tx = Transaction::new_with_payer(&[init_ix_full().unwrap()], Some(&payer.pubkey()));
    tx.sign(&[&payer], last_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // make sure still works after 2x init
    let tx = borrow_donate_check_repaid_tx(&payer, last_blockhash);
    banks_client.process_transaction(tx).await.unwrap();
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn init_fail_insufficient_funds() {
    let pt = ProgramTest::default().add_slumlord_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let insufficient_fund_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        1,
    );
    let mut tx = Transaction::new_with_payer(
        &[insufficient_fund_ix, init_ix_full().unwrap()],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer], last_blockhash);
    // The program logs will end with "success", but the tx actly failed
    // TODO: assert == `TransactionError(InsufficientFundsForRent { account_index: 1 })`
    banks_client.process_transaction(tx).await.unwrap_err();
    banks_client.assert_account_not_exist(SLUMLORD_ID).await;
}

#[tokio::test]
async fn init_fail_no_funds() {
    let pt = ProgramTest::default().add_slumlord_program();
    let (mut banks_client, payer, last_blockhash) = pt.start().await;
    let mut tx = Transaction::new_with_payer(&[init_ix_full().unwrap()], Some(&payer.pubkey()));
    tx.sign(&[&payer], last_blockhash);
    // if slumlord has no lamports at all then the tx succeeds,
    // but the account is not created
    banks_client.process_transaction(tx).await.unwrap();
    banks_client.assert_account_not_exist(SLUMLORD_ID).await;
}

#[tokio::test]
async fn basic() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let tx = borrow_donate_check_repaid_tx(&payer, last_blockhash);

    banks_client.process_transaction(tx).await.unwrap();

    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn basic_repay() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let borrow_ix = borrow_ix(BorrowFreeArgs {
        dst: payer.pubkey(),
    })
    .unwrap();
    let repay_ix = repay_ix(RepayFreeArgs {
        src: payer.pubkey(),
    })
    .unwrap();
    let check_repaid_ix = check_repaid_ix_full().unwrap();

    let mut tx = Transaction::new_with_payer(
        &[borrow_ix, repay_ix, check_repaid_ix],
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
async fn borrow_fail_no_check_repaid() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let borrow_ix = borrow_ix(BorrowFreeArgs {
        dst: payer.pubkey(),
    })
    .unwrap();
    let donate_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        SLUMLORD_LAMPORTS - 1,
    );
    let mut tx = Transaction::new_with_payer(&[borrow_ix, donate_ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], last_blockhash);

    let err = banks_client.process_transaction(tx).await.unwrap_err();

    assert_custom_err(err, SlumlordError::NoSucceedingCheckRepaid);
    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}

#[tokio::test]
async fn borrow_twice_fail() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let borrow_ix = borrow_ix(BorrowFreeArgs {
        dst: payer.pubkey(),
    })
    .unwrap();
    let donate_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        SLUMLORD_LAMPORTS - 1,
    );
    let check_repaid_ix = check_repaid_ix_full().unwrap();
    let mut tx = Transaction::new_with_payer(
        &[borrow_ix.clone(), donate_ix, borrow_ix, check_repaid_ix],
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

#[tokio::test]
async fn insufficient_repay_fail() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let borrow_ix = borrow_ix(BorrowFreeArgs {
        dst: payer.pubkey(),
    })
    .unwrap();
    let insufficient_donate_ix = transfer_ix(
        TransferKeys {
            from: payer.pubkey(),
            to: SLUMLORD_ID,
        },
        SLUMLORD_LAMPORTS - 2,
    );
    let check_repaid_ix = check_repaid_ix_full().unwrap();
    let mut tx = Transaction::new_with_payer(
        &[borrow_ix, insufficient_donate_ix, check_repaid_ix],
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
async fn loan_to_self_fail() {
    let pt = ProgramTest::default()
        .add_slumlord_program()
        .add_slumlord(SLUMLORD_LAMPORTS);

    let (mut banks_client, payer, last_blockhash) = pt.start().await;

    let borrow_ix = borrow_ix(BorrowFreeArgs { dst: SLUMLORD_ID }).unwrap();
    let check_repaid_ix = check_repaid_ix_full().unwrap();
    let mut tx = Transaction::new_with_payer(&[borrow_ix, check_repaid_ix], Some(&payer.pubkey()));
    tx.sign(&[&payer], last_blockhash);

    // The program logs will end with "success", but the tx actly failed
    // TODO: assert == TransactionError(InstructionError(0, UnbalancedInstruction))
    banks_client.process_transaction(tx).await.unwrap_err();

    banks_client
        .assert_slumlord_balance(SLUMLORD_LAMPORTS)
        .await;
    banks_client.assert_slumlord_data_empty().await;
}
