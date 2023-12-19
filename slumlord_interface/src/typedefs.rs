use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, PartialEq, Pod, Copy, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Slumlord {
    pub old_lamports: u64,
}
