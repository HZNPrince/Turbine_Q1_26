use anchor_lang::prelude::*;

mod error;
mod instructions;
mod state;

use instructions::*;
// use state::*;

declare_id!("2map9UzLVbir3eLJ3WVqgDhb9tQ2Dv5W2R7uHmMDmRYg");

#[program]
pub mod anchor_mplxcore {
    use super::*;

    pub fn whitelist_creator(ctx: Context<WhitelistCreator>) -> Result<()> {
        ctx.accounts.whitelist_creator()
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        args: CreateCollectionArgs,
    ) -> Result<()> {
        ctx.accounts.create_collection(args, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<MintNft>) -> Result<()> {
        ctx.accounts.mint_nft()
    }

    pub fn freeze_nft(ctx: Context<FreezeNft>) -> Result<()> {
        ctx.accounts.freeze_nft()
    }

    pub fn thaw_nft(ctx: Context<ThawNft>) -> Result<()> {
        ctx.accounts.thaw_nft()
    }

    pub fn update_nft(ctx: Context<UpdateNft>, new_name: String, new_uri: String) -> Result<()> {
        ctx.accounts.update_nft(new_name, new_uri)
    }
}
