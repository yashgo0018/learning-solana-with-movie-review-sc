use borsh::{BorshDeserialize};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub enum MovieInstruction {
    AddMovieReview {
        title: String,
        rating: u8,
        description: String
    },
    UpdateMovieReview {
        title: String,
        rating: u8,
        description: String
    },
    AddComment {
        comment: String
    }
}

#[derive(BorshDeserialize)]
struct MovieReviewPayload {
    title: String,
    rating: u8,
    description: String
}

#[derive(BorshDeserialize)]
struct CommentPayload {
    comment: String
}

impl MovieInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        Ok(match variant {
            0 => {
                let instruction = MovieReviewPayload::try_from_slice(rest).unwrap();
                Self::AddMovieReview {
                    title: instruction.title,
                    rating: instruction.rating,
                    description: instruction.description,
                }
            },
            1 => {
                let instruction = MovieReviewPayload::try_from_slice(rest).unwrap();
                Self::UpdateMovieReview {
                    title: instruction.title,
                    rating: instruction.rating,
                    description: instruction.description,
                }
            },
            2 => {
                let instruction = CommentPayload::try_from_slice(rest).unwrap();
                Self::AddComment {
                    comment: instruction.comment
                }
            },
            _ => return Err(ProgramError::InvalidInstructionData)
        })
    }
}