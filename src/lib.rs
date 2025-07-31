// use solana_program::{
//     account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
// };
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer,
};
// use solana_system_interface::instruction::transfer;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ErrorCode {
    /// Profit is below the minimum threshold, transaction reverted. （0x64 = 100）
    #[error("Profit is below threshold, transaction reverted.")]
    NotEnoughProfit = 100,
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl TryFrom<u32> for ErrorCode {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(ErrorCode::NotEnoughProfit),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

entrypoint!(process_instruction);

// pub struct Transfer<'info> {
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }
// instruction_data = (min_profit:u64, before_amount: u64)

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let data = instruction_data;

    // 解析参数
    if data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let min_profit = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let before_amount = u64::from_le_bytes(data[8..16].try_into().unwrap());

    let account_info_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut accounts.iter();

    // signer account
    let payer = next_account_info(account_info_iter)?;
    if payer.is_signer != true {
        msg!("Payer must is signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !payer.is_writable {
        msg!("Payer must be writable");
        return Err(ProgramError::InvalidAccountData); // or custom error
    }

    let to_recipient = next_account_info(account_info_iter)?;
    if !to_recipient.is_writable {
        msg!("Recipient must be writable");
        return Err(ProgramError::InvalidAccountData); // or custom error
    }

    // The system program is a required account to invoke a system instruction
    let system_program = next_account_info(account_info_iter)?;

    let after_amount = payer.lamports();
    let profit = after_amount.saturating_sub(before_amount);
    msg!(
        "{} - {} = {} [{}]",
        after_amount,
        before_amount,
        after_amount as i64 - before_amount as i64,
        min_profit
    );
    if profit < min_profit {
        msg!("No profitable arbitrage found");
        return Err(ErrorCode::NotEnoughProfit.into());
    }
    // msg!("Report: profit={}", profit);

    // let fee = (profit as f64 * 0.1) as u64;
    // let ix = transfer(payer.key, to_recipient.key, fee);
    // invoke(
    //     &ix,
    //     &[payer.clone(), to_recipient.clone(), system_program.clone()],
    // )?;
    // msg!("Report: profit={}", profit);

    Ok(())
}
