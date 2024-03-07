use solana_program::account_info::AccountInfo;
use solana_program::entrypoint;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use crate::instructions::MovieInstruction;
use crate::processors::{add_comment, add_movie_review, update_movie_review};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
    let instruction = MovieInstruction::unpack(&instruction_data)?;

    match instruction {
        MovieInstruction::AddMovieReview {title, rating, description} => {
            add_movie_review(program_id, accounts, title, rating, description)
        }
        MovieInstruction::UpdateMovieReview {title, rating, description} => {
            update_movie_review(program_id, accounts, title, rating, description)
        }
        MovieInstruction::AddComment {comment} => {
            add_comment(program_id, accounts, comment)
        }
    }
}
