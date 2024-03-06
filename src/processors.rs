use borsh::BorshSerialize;
use solana_program::{
    program::invoke_signed,
    entrypoint::ProgramResult,
    account_info::{AccountInfo, next_account_info},
    pubkey::Pubkey,
    msg,
    system_instruction,
    borsh1::try_from_slice_unchecked,
    sysvar::{rent::Rent, Sysvar},
};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::IsInitialized;
use crate::state::MovieAccountState;
// inside processor.rs
use crate::error::ReviewError;

pub fn add_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String
) -> ProgramResult {

    // Get Account iterator
    let account_info_iter = &mut accounts.iter();

    // Get accounts
    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive PDA
    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref(),], program_id);

    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if rating > 5 || rating < 1 {
        msg!("Rating cannot be higher than 5 or lower than 1");
        return Err(ReviewError::InvalidRating.into());
    }

    // Calculate account size required
    let account_len: usize = 1000;

    let total_len: usize = 1 + 1 + (4 + title.len()) + (4 + description.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into())
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    // Create the account
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[initializer.clone(), pda_account.clone(), system_program.clone()],
        &[&[initializer.key.as_ref(), title.as_bytes().as_ref(), &[bump_seed]]],
    )?;

    msg!("PDA created: {}", pda);

    msg!("unpacking state account");
    let mut account_data = try_from_slice_unchecked::<MovieAccountState>(&pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;
    account_data.is_initialized = true;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn update_movie_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    rating: u8,
    description: String
) -> ProgramResult {
    let account_iterator = &mut accounts.iter();

    let initializer = next_account_info(account_iterator)?;
    let pda_account = next_account_info(account_iterator)?;

    if pda_account.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner)
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref(),], program_id);

    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if rating > 5 || rating < 1 {
        msg!("Rating cannot be higher than 5");
        return Err(ReviewError::InvalidRating.into())
    }

    let total_len: usize = 1 + 1 + (4 + title.len()) + (4 + description.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into())
    }


    let mut account_data = try_from_slice_unchecked::<MovieAccountState>(&pda_account.data.borrow())?;

    if !account_data.is_initialized {
        msg!("old review not found");
        return Err(ReviewError::InvalidPDA.into());
    }

    account_data.rating = rating;
    account_data.description = description;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}