use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token, Transfer};

pub static ADMIN_PUBKEY: Pubkey = Pubkey::new_from_array([
    211, 230, 237,   3, 189, 105, 218,  45,
    202, 238, 106,  75,  18,  51,  59,  37,
     66, 219, 112, 176,  97,  61, 156, 239,
    120, 188, 212, 132, 136, 172, 197,  67
  ]);
pub static TOKEN_MINT: Pubkey = Pubkey::new_from_array([
    6, 130,   0, 105,   8, 118,  3,  61,
   44, 223, 115, 187, 103, 195, 82, 229,
  174, 109, 234,  13,  50,  53, 35, 150,
   76,  82, 145, 144,  39, 202, 43, 170
]);
pub static STAKE_AMOUNT: u64 = 5000_00000000;

declare_id!("4wK5hPs97SX8Mb3fAwbz6nsBkPw6u7vPxGvUsc1jWCh6");

#[program]
pub mod simple_stake {
    use super::*;

    pub fn create_token_pool(_ctx: Context<CreateTokenPool>) -> Result<()> {
        // Everything is already done by Anchor
        Ok(())
    }

    pub fn stake_tokens(ctx: Context<StakeTokens>) -> Result<()> {
        // create info pda account (done by Anchor)
        // transfer fixed amount of tokens from user account to common stake account
        // TODO move cpi initialization into impl
        let cpi_token = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_tokens.to_account_info(),
            to: ctx.accounts.pool_tokens.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_token, cpi_accounts);
        anchor_spl::token::transfer(cpi_context, STAKE_AMOUNT)?;
        // set staked flag in info pda account
        let info = &mut ctx.accounts.info;
        info.staked_amount = STAKE_AMOUNT;
        info.user = ctx.accounts.user.key();
        Ok(())
    }

    pub fn unstake_tokens(ctx: Context<UnstakeTokens>) -> Result<()> {
        // transfer fixed amount of tokens from common stake account back to user account
        // TODO move cpi initialization into impl
        let cpi_token = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_tokens.to_account_info(),
            to: ctx.accounts.user_tokens.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        };
        let bump = *ctx.bumps.get("pool_authority").unwrap();
        let seeds: &[&[&[u8]]] = &[&[b"pool-authority", &[bump]]];
        let cpi_context = CpiContext::new(cpi_token, cpi_accounts)
            .with_signer(seeds);
        anchor_spl::token::transfer(cpi_context, STAKE_AMOUNT)?;
        // info account will be closed by Anchor
        Ok(())
    }

    pub fn stake_tokens_from(ctx: Context<StakeTokensFrom>, user: Pubkey) -> Result<()> {
        // create info pda account (done by Anchor)
        // transfer fixed amount of tokens from user account to common stake account
        // TODO move cpi initialization into impl
        let cpi_token = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_tokens.to_account_info(),
            to: ctx.accounts.pool_tokens.to_account_info(),
            authority: ctx.accounts.user_token_authority.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_token, cpi_accounts);
        anchor_spl::token::transfer(cpi_context, STAKE_AMOUNT)?;
        // set staked flag in info pda account
        let info = &mut ctx.accounts.info;
        info.staked_amount = STAKE_AMOUNT;
        info.user = user;
        Ok(())
    }

    pub fn lock_stake(ctx: Context<LockUnlockStake>) -> Result<()> {
        let info = &mut ctx.accounts.info;
        info.is_locked = true;
        Ok(())
    }

    pub fn unlock_stake(ctx: Context<LockUnlockStake>) -> Result<()> {
        let info = &mut ctx.accounts.info;
        info.is_locked = false;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"stake-info", user.key().as_ref()],
        bump
    )]
    pub info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub user_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_tokens: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeTokensFrom<'info> {
    #[account(
        mut,
        address = ADMIN_PUBKEY,
    )]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [b"stake-info", admin.key().as_ref()],
        bump
    )]
    pub info: Account<'info, StakeInfo>,
    pub user_token_authority: Signer<'info>,
    #[account(mut)]
    pub user_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_tokens: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateTokenPool<'info> {
    #[account(
        mut,
        address = ADMIN_PUBKEY,
    )]
    pub admin: Signer<'info>,
    /// CHECK: only used as a signing PDA
    #[account(
        seeds = [b"pool-authority"],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        token::mint = mint,
        token::authority = pool_authority,
        seeds = [b"pool-tokens"],
        bump
    )]
    pub pool_tokens: Account<'info, TokenAccount>,
    #[account(
        address = TOKEN_MINT,
    )]
    pub mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        has_one = user,
        seeds = [b"stake-info", user.key().as_ref()], bump,
        constraint = !info.is_locked,
        close = user
    )]
    pub info: Account<'info, StakeInfo>,
    #[account(mut)]
    pub user_tokens: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_tokens: Account<'info, TokenAccount>,
    /// CHECK: only used as a signing PDA
    #[account(
        seeds = [b"pool-authority"],
        bump
    )]
    pub pool_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct LockUnlockStake<'info> {
    #[account(
        address = ADMIN_PUBKEY,
    )]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub info: Account<'info, StakeInfo>,
}

#[account]
#[derive(Default)]
pub struct StakeInfo {
    user: Pubkey,
    staked_amount: u64,
    is_locked: bool,
    stake_type: u8,
}

#[error_code]
pub enum SimpleStakeError {
    #[msg("Couldn't unstake because stake is locked")]
    StakeAccountLocked,
}
