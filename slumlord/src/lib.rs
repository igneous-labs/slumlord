use sanctum_misc_utils::{
    load_accounts, log_and_return_acc_privilege_err, log_and_return_wrong_acc_err,
};
use sanctum_system_program_lib::{
    assign_invoke_signed, transfer_direct_increment, transfer_invoke, ResizableAccount,
    TransferAccounts,
};
use slumlord_interface::{
    borrow_verify_account_keys, borrow_verify_account_privileges, check_repaid_verify_account_keys,
    check_repaid_verify_account_privileges, init_verify_account_keys,
    init_verify_account_privileges, repay_verify_account_keys, repay_verify_account_privileges,
    BorrowAccounts, CheckRepaidAccounts, InitAccounts, RepayAccounts, SlumlordError,
    SlumlordProgramIx, CHECK_REPAID_IX_DISCM,
};
use slumlord_lib::{
    program::{SLUMLORD_BUMP, SLUMLORD_SEED},
    try_slumlord_mut, BorrowFreeArgs, LoanActiveSlumlordAccount, RepayFreeArgs, CHECK_REPAID_KEYS,
    INIT_KEYS, SLUMLORD_ACCOUNT_LEN,
};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::Instruction,
    program_error::{PrintProgramError, ProgramError},
    pubkey::Pubkey,
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
};

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if *program_id != slumlord_lib::program::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    let ix = SlumlordProgramIx::deserialize(instruction_data)?;
    solana_program::msg!("{:?}", ix);

    let res: ProgramResult = match ix {
        SlumlordProgramIx::Init => process_init(accounts),
        SlumlordProgramIx::Borrow => process_borrow(accounts),
        SlumlordProgramIx::Repay => process_repay(accounts),
        SlumlordProgramIx::CheckRepaid => process_check_repaid(accounts),
    };
    if let Err(e) = res.as_ref() {
        e.print::<SlumlordError>();
    }
    res
}

/// Assign slumlord PDA to slumlord program.
///
/// Permissionless, called once.
///
/// Can call multiple times: system_program::assign() will be a no-op
///
/// Pre-requisites:
/// - slumlord PDA should be funded with enough for rent-exempt 0.
///   These funds are locked in there and serve as the flash loan amount
fn process_init(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts: InitAccounts = load_accounts(accounts)?;

    init_verify_account_keys(accounts, INIT_KEYS).map_err(log_and_return_wrong_acc_err)?;
    init_verify_account_privileges(accounts).map_err(log_and_return_acc_privilege_err)?;

    assign_invoke_signed(
        accounts.slumlord,
        slumlord_lib::program::ID,
        &[&[SLUMLORD_SEED, &[SLUMLORD_BUMP]]],
    )?;

    Ok(())
}

/// Flash borrows `slumlord_balance - 1` lamports from slumlord account to
/// specified `dst` account
fn process_borrow(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts: BorrowAccounts = load_accounts(accounts)?;

    let free_args = BorrowFreeArgs {
        dst: *accounts.dst.key,
    };
    borrow_verify_account_keys(accounts, free_args.resolve())
        .map_err(log_and_return_wrong_acc_err)?;
    borrow_verify_account_privileges(accounts).map_err(log_and_return_acc_privilege_err)?;

    let curr_ix_idx: usize = load_current_index_checked(accounts.instructions)?.into();
    let mut next_ix_idx = curr_ix_idx;
    loop {
        next_ix_idx = next_ix_idx
            .checked_add(1)
            .ok_or(ProgramError::InvalidInstructionData)?;
        let next_ix = load_instruction_at_checked(next_ix_idx, accounts.instructions)
            .map_err(|_| SlumlordError::NoSucceedingCheckRepaid)?;
        if is_check_repaid_ix(&next_ix) {
            break;
        }
    }

    if !accounts.slumlord.data_is_empty() {
        return Err(SlumlordError::BorrowAlreadyActive.into());
    }

    let slumlord_lamports = accounts.slumlord.lamports();

    {
        accounts.slumlord.extend_to(SLUMLORD_ACCOUNT_LEN)?;
        let mut slumlord_data = accounts.slumlord.try_borrow_mut_data()?;
        let slumlord = try_slumlord_mut(&mut slumlord_data)?;
        slumlord.old_lamports = slumlord_lamports;
    }

    let borrow_lamports = slumlord_lamports
        .checked_sub(1)
        .ok_or(ProgramError::InsufficientFunds)?;

    transfer_direct_increment(
        TransferAccounts {
            from: accounts.slumlord,
            to: accounts.dst,
        },
        borrow_lamports,
    )?;

    Ok(())
}

fn is_check_repaid_ix(ix: &Instruction) -> bool {
    let discm = match ix.data.first() {
        Some(d) => d,
        None => return false,
    };
    if *discm != CHECK_REPAID_IX_DISCM {
        return false;
    }
    if ix.program_id != slumlord_lib::program::ID {
        return false;
    }
    true
}

/// Transfer the outstanding loan amount from the `src` system_account
/// to the slumlord account.
///
/// This is a util ix for borrowers to easily repay the loan amount without
/// having to calculate it prior.
fn process_repay(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts: RepayAccounts = load_accounts(accounts)?;

    let free_args = RepayFreeArgs {
        src: *accounts.src.key,
    };
    repay_verify_account_keys(accounts, free_args.resolve())
        .map_err(log_and_return_wrong_acc_err)?;
    repay_verify_account_privileges(accounts).map_err(log_and_return_acc_privilege_err)?;

    transfer_invoke(
        TransferAccounts {
            from: accounts.src,
            to: accounts.slumlord,
        },
        accounts.slumlord.curr_loan_lamports_outstanding()?,
    )?;

    Ok(())
}

/// Verifies that the flash loan has been completely repaid,
/// ending the flash loan
fn process_check_repaid(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts: CheckRepaidAccounts = load_accounts(accounts)?;

    check_repaid_verify_account_keys(accounts, CHECK_REPAID_KEYS)
        .map_err(log_and_return_wrong_acc_err)?;
    check_repaid_verify_account_privileges(accounts).map_err(log_and_return_acc_privilege_err)?;

    let slumlord_lamports = accounts.slumlord.lamports();
    let min_expected_slumlord_lamports = accounts.slumlord.old_lamports()?;

    if slumlord_lamports < min_expected_slumlord_lamports {
        return Err(SlumlordError::InsufficientRepay.into());
    }

    accounts.slumlord.shrink_to(0)?;

    Ok(())
}
