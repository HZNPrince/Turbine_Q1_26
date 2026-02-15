use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

declare_id!("D1AtSk1MYWSv6PCg2GzpaDwoKNxtXxKKeToFuSs5DYq2");

#[program]
pub mod quadratic_funding {
    use super::*;

    pub fn init_dao(ctx: Context<InitDao>, name: String) -> Result<()> {
        ctx.accounts.init_dao(name, &ctx.bumps)
    }

    pub fn init_proposal(ctx: Context<InitProposal>, metadata: String) -> Result<()> {
        instructions::init_proposal(ctx, metadata)
    }

    pub fn cast_vote(ctx: Context<CastVote>, vote_type: u8) -> Result<()> {
        instructions::cast_vote(ctx, vote_type)
    }
}
