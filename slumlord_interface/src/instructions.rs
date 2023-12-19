use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum SlumlordProgramIx {
    Init,
    Borrow,
    Repay,
    CheckRepaid,
}
impl SlumlordProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm_buf = [0u8; 1];
        reader.read_exact(&mut maybe_discm_buf)?;
        let maybe_discm = maybe_discm_buf[0];
        match maybe_discm {
            INIT_IX_DISCM => Ok(Self::Init),
            BORROW_IX_DISCM => Ok(Self::Borrow),
            REPAY_IX_DISCM => Ok(Self::Repay),
            CHECK_REPAID_IX_DISCM => Ok(Self::CheckRepaid),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("discm {:?} not found", maybe_discm),
            )),
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::Init => writer.write_all(&[INIT_IX_DISCM]),
            Self::Borrow => writer.write_all(&[BORROW_IX_DISCM]),
            Self::Repay => writer.write_all(&[REPAY_IX_DISCM]),
            Self::CheckRepaid => writer.write_all(&[CHECK_REPAID_IX_DISCM]),
        }
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub const INIT_IX_ACCOUNTS_LEN: usize = 2;
#[derive(Copy, Clone, Debug)]
pub struct InitAccounts<'me, 'info> {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: &'me AccountInfo<'info>,
    ///System Program
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug)]
pub struct InitKeys {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: Pubkey,
    ///System Program
    pub system_program: Pubkey,
}
impl From<InitAccounts<'_, '_>> for InitKeys {
    fn from(accounts: InitAccounts) -> Self {
        Self {
            slumlord: *accounts.slumlord.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<InitKeys> for [AccountMeta; INIT_IX_ACCOUNTS_LEN] {
    fn from(keys: InitKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.slumlord,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INIT_IX_ACCOUNTS_LEN]> for InitKeys {
    fn from(pubkeys: [Pubkey; INIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: pubkeys[0],
            system_program: pubkeys[1],
        }
    }
}
impl<'info> From<InitAccounts<'_, 'info>> for [AccountInfo<'info>; INIT_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitAccounts<'_, 'info>) -> Self {
        [accounts.slumlord.clone(), accounts.system_program.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INIT_IX_ACCOUNTS_LEN]>
    for InitAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; INIT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: &arr[0],
            system_program: &arr[1],
        }
    }
}
pub const INIT_IX_DISCM: u8 = 0u8;
#[derive(Clone, Debug, PartialEq)]
pub struct InitIxData;
impl InitIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm_buf = [0u8; 1];
        reader.read_exact(&mut maybe_discm_buf)?;
        let maybe_discm = maybe_discm_buf[0];
        if maybe_discm != INIT_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    INIT_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&[INIT_IX_DISCM])
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn init_ix<K: Into<InitKeys>>(accounts: K) -> std::io::Result<Instruction> {
    let keys: InitKeys = accounts.into();
    let metas: [AccountMeta; INIT_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id: crate::ID,
        accounts: Vec::from(metas),
        data: InitIxData.try_to_vec()?,
    })
}
pub fn init_invoke<'info>(accounts: InitAccounts<'_, 'info>) -> ProgramResult {
    let ix = init_ix(accounts)?;
    let account_info: [AccountInfo<'info>; INIT_IX_ACCOUNTS_LEN] = accounts.into();
    invoke(&ix, &account_info)
}
pub fn init_invoke_signed<'info>(
    accounts: InitAccounts<'_, 'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = init_ix(accounts)?;
    let account_info: [AccountInfo<'info>; INIT_IX_ACCOUNTS_LEN] = accounts.into();
    invoke_signed(&ix, &account_info, seeds)
}
pub fn init_verify_account_keys(
    accounts: InitAccounts<'_, '_>,
    keys: InitKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (accounts.slumlord.key, &keys.slumlord),
        (accounts.system_program.key, &keys.system_program),
    ] {
        if actual != expected {
            return Err((*actual, *expected));
        }
    }
    Ok(())
}
pub fn init_verify_account_privileges<'me, 'info>(
    accounts: InitAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.slumlord] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub const BORROW_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct BorrowAccounts<'me, 'info> {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: &'me AccountInfo<'info>,
    ///The destination account to lend SOL to
    pub dst: &'me AccountInfo<'info>,
    ///Instructions sysvar
    pub instructions: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug)]
pub struct BorrowKeys {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: Pubkey,
    ///The destination account to lend SOL to
    pub dst: Pubkey,
    ///Instructions sysvar
    pub instructions: Pubkey,
}
impl From<BorrowAccounts<'_, '_>> for BorrowKeys {
    fn from(accounts: BorrowAccounts) -> Self {
        Self {
            slumlord: *accounts.slumlord.key,
            dst: *accounts.dst.key,
            instructions: *accounts.instructions.key,
        }
    }
}
impl From<BorrowKeys> for [AccountMeta; BORROW_IX_ACCOUNTS_LEN] {
    fn from(keys: BorrowKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.slumlord,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.dst,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.instructions,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; BORROW_IX_ACCOUNTS_LEN]> for BorrowKeys {
    fn from(pubkeys: [Pubkey; BORROW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: pubkeys[0],
            dst: pubkeys[1],
            instructions: pubkeys[2],
        }
    }
}
impl<'info> From<BorrowAccounts<'_, 'info>> for [AccountInfo<'info>; BORROW_IX_ACCOUNTS_LEN] {
    fn from(accounts: BorrowAccounts<'_, 'info>) -> Self {
        [
            accounts.slumlord.clone(),
            accounts.dst.clone(),
            accounts.instructions.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; BORROW_IX_ACCOUNTS_LEN]>
    for BorrowAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; BORROW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: &arr[0],
            dst: &arr[1],
            instructions: &arr[2],
        }
    }
}
pub const BORROW_IX_DISCM: u8 = 1u8;
#[derive(Clone, Debug, PartialEq)]
pub struct BorrowIxData;
impl BorrowIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm_buf = [0u8; 1];
        reader.read_exact(&mut maybe_discm_buf)?;
        let maybe_discm = maybe_discm_buf[0];
        if maybe_discm != BORROW_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    BORROW_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&[BORROW_IX_DISCM])
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn borrow_ix<K: Into<BorrowKeys>>(accounts: K) -> std::io::Result<Instruction> {
    let keys: BorrowKeys = accounts.into();
    let metas: [AccountMeta; BORROW_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id: crate::ID,
        accounts: Vec::from(metas),
        data: BorrowIxData.try_to_vec()?,
    })
}
pub fn borrow_invoke<'info>(accounts: BorrowAccounts<'_, 'info>) -> ProgramResult {
    let ix = borrow_ix(accounts)?;
    let account_info: [AccountInfo<'info>; BORROW_IX_ACCOUNTS_LEN] = accounts.into();
    invoke(&ix, &account_info)
}
pub fn borrow_invoke_signed<'info>(
    accounts: BorrowAccounts<'_, 'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = borrow_ix(accounts)?;
    let account_info: [AccountInfo<'info>; BORROW_IX_ACCOUNTS_LEN] = accounts.into();
    invoke_signed(&ix, &account_info, seeds)
}
pub fn borrow_verify_account_keys(
    accounts: BorrowAccounts<'_, '_>,
    keys: BorrowKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (accounts.slumlord.key, &keys.slumlord),
        (accounts.dst.key, &keys.dst),
        (accounts.instructions.key, &keys.instructions),
    ] {
        if actual != expected {
            return Err((*actual, *expected));
        }
    }
    Ok(())
}
pub fn borrow_verify_account_privileges<'me, 'info>(
    accounts: BorrowAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.slumlord, accounts.dst] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub const REPAY_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct RepayAccounts<'me, 'info> {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: &'me AccountInfo<'info>,
    ///The system account paying the outstanding flash loan
    pub src: &'me AccountInfo<'info>,
    ///System Program
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug)]
pub struct RepayKeys {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: Pubkey,
    ///The system account paying the outstanding flash loan
    pub src: Pubkey,
    ///System Program
    pub system_program: Pubkey,
}
impl From<RepayAccounts<'_, '_>> for RepayKeys {
    fn from(accounts: RepayAccounts) -> Self {
        Self {
            slumlord: *accounts.slumlord.key,
            src: *accounts.src.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<RepayKeys> for [AccountMeta; REPAY_IX_ACCOUNTS_LEN] {
    fn from(keys: RepayKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.slumlord,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.src,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; REPAY_IX_ACCOUNTS_LEN]> for RepayKeys {
    fn from(pubkeys: [Pubkey; REPAY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: pubkeys[0],
            src: pubkeys[1],
            system_program: pubkeys[2],
        }
    }
}
impl<'info> From<RepayAccounts<'_, 'info>> for [AccountInfo<'info>; REPAY_IX_ACCOUNTS_LEN] {
    fn from(accounts: RepayAccounts<'_, 'info>) -> Self {
        [
            accounts.slumlord.clone(),
            accounts.src.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; REPAY_IX_ACCOUNTS_LEN]>
    for RepayAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; REPAY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: &arr[0],
            src: &arr[1],
            system_program: &arr[2],
        }
    }
}
pub const REPAY_IX_DISCM: u8 = 2u8;
#[derive(Clone, Debug, PartialEq)]
pub struct RepayIxData;
impl RepayIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm_buf = [0u8; 1];
        reader.read_exact(&mut maybe_discm_buf)?;
        let maybe_discm = maybe_discm_buf[0];
        if maybe_discm != REPAY_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    REPAY_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&[REPAY_IX_DISCM])
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn repay_ix<K: Into<RepayKeys>>(accounts: K) -> std::io::Result<Instruction> {
    let keys: RepayKeys = accounts.into();
    let metas: [AccountMeta; REPAY_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id: crate::ID,
        accounts: Vec::from(metas),
        data: RepayIxData.try_to_vec()?,
    })
}
pub fn repay_invoke<'info>(accounts: RepayAccounts<'_, 'info>) -> ProgramResult {
    let ix = repay_ix(accounts)?;
    let account_info: [AccountInfo<'info>; REPAY_IX_ACCOUNTS_LEN] = accounts.into();
    invoke(&ix, &account_info)
}
pub fn repay_invoke_signed<'info>(
    accounts: RepayAccounts<'_, 'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = repay_ix(accounts)?;
    let account_info: [AccountInfo<'info>; REPAY_IX_ACCOUNTS_LEN] = accounts.into();
    invoke_signed(&ix, &account_info, seeds)
}
pub fn repay_verify_account_keys(
    accounts: RepayAccounts<'_, '_>,
    keys: RepayKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (accounts.slumlord.key, &keys.slumlord),
        (accounts.src.key, &keys.src),
        (accounts.system_program.key, &keys.system_program),
    ] {
        if actual != expected {
            return Err((*actual, *expected));
        }
    }
    Ok(())
}
pub fn repay_verify_account_privileges<'me, 'info>(
    accounts: RepayAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.slumlord, accounts.src] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    for should_be_signer in [accounts.src] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub const CHECK_REPAID_IX_ACCOUNTS_LEN: usize = 1;
#[derive(Copy, Clone, Debug)]
pub struct CheckRepaidAccounts<'me, 'info> {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug)]
pub struct CheckRepaidKeys {
    ///The slumlord PDA ["slumlord"]
    pub slumlord: Pubkey,
}
impl From<CheckRepaidAccounts<'_, '_>> for CheckRepaidKeys {
    fn from(accounts: CheckRepaidAccounts) -> Self {
        Self {
            slumlord: *accounts.slumlord.key,
        }
    }
}
impl From<CheckRepaidKeys> for [AccountMeta; CHECK_REPAID_IX_ACCOUNTS_LEN] {
    fn from(keys: CheckRepaidKeys) -> Self {
        [AccountMeta {
            pubkey: keys.slumlord,
            is_signer: false,
            is_writable: true,
        }]
    }
}
impl From<[Pubkey; CHECK_REPAID_IX_ACCOUNTS_LEN]> for CheckRepaidKeys {
    fn from(pubkeys: [Pubkey; CHECK_REPAID_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            slumlord: pubkeys[0],
        }
    }
}
impl<'info> From<CheckRepaidAccounts<'_, 'info>>
    for [AccountInfo<'info>; CHECK_REPAID_IX_ACCOUNTS_LEN]
{
    fn from(accounts: CheckRepaidAccounts<'_, 'info>) -> Self {
        [accounts.slumlord.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CHECK_REPAID_IX_ACCOUNTS_LEN]>
    for CheckRepaidAccounts<'me, 'info>
{
    fn from(arr: &'me [AccountInfo<'info>; CHECK_REPAID_IX_ACCOUNTS_LEN]) -> Self {
        Self { slumlord: &arr[0] }
    }
}
pub const CHECK_REPAID_IX_DISCM: u8 = 3u8;
#[derive(Clone, Debug, PartialEq)]
pub struct CheckRepaidIxData;
impl CheckRepaidIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm_buf = [0u8; 1];
        reader.read_exact(&mut maybe_discm_buf)?;
        let maybe_discm = maybe_discm_buf[0];
        if maybe_discm != CHECK_REPAID_IX_DISCM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "discm does not match. Expected: {:?}. Received: {:?}",
                    CHECK_REPAID_IX_DISCM, maybe_discm
                ),
            ));
        }
        Ok(Self)
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&[CHECK_REPAID_IX_DISCM])
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn check_repaid_ix<K: Into<CheckRepaidKeys>>(accounts: K) -> std::io::Result<Instruction> {
    let keys: CheckRepaidKeys = accounts.into();
    let metas: [AccountMeta; CHECK_REPAID_IX_ACCOUNTS_LEN] = keys.into();
    Ok(Instruction {
        program_id: crate::ID,
        accounts: Vec::from(metas),
        data: CheckRepaidIxData.try_to_vec()?,
    })
}
pub fn check_repaid_invoke<'info>(accounts: CheckRepaidAccounts<'_, 'info>) -> ProgramResult {
    let ix = check_repaid_ix(accounts)?;
    let account_info: [AccountInfo<'info>; CHECK_REPAID_IX_ACCOUNTS_LEN] = accounts.into();
    invoke(&ix, &account_info)
}
pub fn check_repaid_invoke_signed<'info>(
    accounts: CheckRepaidAccounts<'_, 'info>,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let ix = check_repaid_ix(accounts)?;
    let account_info: [AccountInfo<'info>; CHECK_REPAID_IX_ACCOUNTS_LEN] = accounts.into();
    invoke_signed(&ix, &account_info, seeds)
}
pub fn check_repaid_verify_account_keys(
    accounts: CheckRepaidAccounts<'_, '_>,
    keys: CheckRepaidKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [(accounts.slumlord.key, &keys.slumlord)] {
        if actual != expected {
            return Err((*actual, *expected));
        }
    }
    Ok(())
}
pub fn check_repaid_verify_account_privileges<'me, 'info>(
    accounts: CheckRepaidAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.slumlord] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
