// use solana_program::{
//     account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
// };
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    // program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
// use solana_sdk::system_instruction;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ErrorCode {
    /// Profit is below the minimum threshold, transaction reverted.
    #[error("Profit is below threshold, transaction reverted.")]
    NotEnoughProfit,
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

/// params
// pub struct Transfer<'info> {
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }

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
    // The system program is a required account to invoke a system instruction
    // let system_program = next_account_info(account_info_iter)?;

    // 读取Token账户数据中的amount
    let user_token_data = payer.try_borrow_data()?;

    // SPL Token Account结构，amount在偏移44~52字节
    let after_amount = {
        let amount_bytes: [u8; 8] = user_token_data[64..72].try_into().unwrap();
        u64::from_le_bytes(amount_bytes)
    };

    let profit = after_amount.saturating_sub(before_amount);
    if profit < min_profit {
        msg!("No profitable arbitrage found");
        return Err(ErrorCode::NotEnoughProfit.into());
    }
    msg!("Report: profit={}", profit);

    // let fee = 1u64;
    // // let fee_recipient = pubkey!("BHEJ4zEvjy7Py9C5GBteZk7DBHzvrdiqGt3i3yXd64sv");
    // // transfer(from_pubkey, to_pubkey, lamports);
    // let fee = 1u64;
    // let to_recipient = pubkey!("BHEJ4zEvjy7Py9C5GBteZk7DBHzvrdiqGt3i3yXd64sv");
    // let ix = system_instruction::transfer(payer.key, to_recipient, fee);

    // // 调用系统程序转账
    // invoke(
    //     &ix,
    //     &[payer.clone(), to_recipient.clone(), system_program.clone()],
    // )?;
    // msg!("Report: profit={}", profit);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{account_info::AccountInfo, clock::Epoch};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn create_account_info<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        use solana_program::{account_info::AccountInfo, clock::Epoch};

        AccountInfo::new(
            key,
            true,
            true,
            lamports,
            data,
            owner,
            false,
            Epoch::default(),
        )
    }

    #[test]
    fn test_enough_profit() {
        use solana_program::pubkey::Pubkey;

        let key = Pubkey::new_unique();
        let owner = Pubkey::default();

        let mut lamports: u64 = 0;
        let mut data = vec![0u8; 72];
        data[64..72].copy_from_slice(&200u64.to_le_bytes());

        // ✅ 把 key 和 owner 的引用传进去
        let payer = create_account_info(&key, &mut lamports, &mut data, &owner);

        let accounts = vec![payer];

        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&50u64.to_le_bytes());
        instruction_data.extend_from_slice(&100u64.to_le_bytes());

        let res = process_instruction(&Pubkey::new_unique(), &accounts, &instruction_data);

        assert!(res.is_ok());
    }
}
