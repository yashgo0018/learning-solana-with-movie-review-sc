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
use crate::state::{MovieAccountState, MovieComment, MovieCommentCounter};
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
    let pda_counter = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive PDA
    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref(),], program_id);
    let (counter, counter_bump) = Pubkey::find_program_address(&[pda.as_ref(), "comment_counter".as_ref()], program_id);

    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if counter != *pda_counter.key {
        msg!("Invalid seeds for Comment Counter PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if rating > 5 || rating < 1 {
        msg!("Rating cannot be higher than 5 or lower than 1");
        return Err(ReviewError::InvalidRating.into());
    }

    // Calculate account size required
    let account_len: usize = 1000;

    let total_len: usize = MovieAccountState::get_account_size(title.clone(), description.clone());
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

    account_data.discriminator = MovieAccountState::DISCRIMINATOR.to_string();
    account_data.is_initialized = true;
    account_data.title = title;
    account_data.rating = rating;
    account_data.description = description;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    // create the comment counter account
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(MovieCommentCounter::SIZE);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_counter.key,
            rent_lamports,
            MovieCommentCounter::SIZE.try_into().unwrap(),
            program_id,
        ),
        &[initializer.clone(), pda_counter.clone(), system_program.clone()],
        &[&[pda.as_ref(), "comment_counter".as_ref(), &[counter_bump]]],
    )?;

    let mut counter_data = try_from_slice_unchecked::<MovieCommentCounter>(&pda_counter.data.borrow()).unwrap();

    if counter_data.is_initialized() {
        msg!("Comment Counter Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter_data.discriminator = MovieCommentCounter::DISCRIMINATOR.to_string();
    counter_data.counter = 0;
    counter_data.is_initialized = true;

    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

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

    if account_data.discriminator != MovieAccountState::DISCRIMINATOR {
        msg!("invalid pda provided");
        return Err(ReviewError::InvalidPDA.into());
    }

    account_data.rating = rating;
    account_data.description = description;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn add_comment(program_id: &Pubkey, accounts: &[AccountInfo], comment: String) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let review_pda_account = next_account_info(accounts_iter)?;
    let comment_counter_pda_account = next_account_info(accounts_iter)?;
    let comment_pda_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // check if both the PDA are correct
    // let comment_counter_pda = Pubkey::find_program_address(&[])

    let mut counter_data = try_from_slice_unchecked::<MovieCommentCounter>(&comment_counter_pda_account.data.borrow()).unwrap();
    if !counter_data.is_initialized() {
        msg!("comment counter is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    counter_data.counter+= counter_data.counter;

    let comment_id = counter_data.counter;

    counter_data.serialize(&mut &mut comment_counter_pda_account.data.borrow_mut()[..])?;

    // create comment account
    let account_len = MovieComment::get_account_size(comment.clone());

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(&[review_pda_account.key.as_ref(), comment_id.to_be_bytes().as_ref()], program_id);

    if pda != *comment_pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into())
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            comment_pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id
        ),
        &[initializer.clone(), comment_pda_account.clone(), system_program.clone()],
        &[&[review_pda_account.key.as_ref(), comment_id.to_be_bytes().as_ref(), &[bump_seed]]]
    )?;

    let mut comment_data = try_from_slice_unchecked::<MovieComment>(&comment_pda_account.data.borrow())?;

    if comment_data.is_initialized() {
        msg!("Comment Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    comment_data.discriminator = MovieComment::DISCRIMINATOR.to_string();
    comment_data.id = comment_id;
    comment_data.commenter = *initializer.key;
    comment_data.comment = comment;
    comment_data.is_initialized = true;
    comment_data.review = *review_pda_account.key;

    comment_data.serialize(&mut &mut comment_pda_account.data.borrow_mut()[..])?;

    Ok(())
}
