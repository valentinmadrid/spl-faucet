#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use crate::{id, seahorse_util::*};
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::{cell::RefCell, rc::Rc};

#[account]
#[derive(Debug)]
pub struct Faucet {
    pub bump: u8,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub max_withdraw: u64,
    pub decimals: u64,
}

impl<'info, 'entrypoint> Faucet {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedFaucet<'info, 'entrypoint>> {
        let bump = account.bump;
        let mint = account.mint.clone();
        let owner = account.owner.clone();
        let max_withdraw = account.max_withdraw;
        let decimals = account.decimals;

        Mutable::new(LoadedFaucet {
            __account__: account,
            __programs__: programs_map,
            bump,
            mint,
            owner,
            max_withdraw,
            decimals,
        })
    }

    pub fn store(loaded: Mutable<LoadedFaucet>) {
        let mut loaded = loaded.borrow_mut();
        let bump = loaded.bump;

        loaded.__account__.bump = bump;

        let mint = loaded.mint.clone();

        loaded.__account__.mint = mint;

        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let max_withdraw = loaded.max_withdraw;

        loaded.__account__.max_withdraw = max_withdraw;

        let decimals = loaded.decimals;

        loaded.__account__.decimals = decimals;
    }
}

#[derive(Debug)]
pub struct LoadedFaucet<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Faucet>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub bump: u8,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub max_withdraw: u64,
    pub decimals: u64,
}

#[account]
#[derive(Debug)]
pub struct Withdrawer {
    pub owner: Pubkey,
    pub last_withdraw: i64,
}

impl<'info, 'entrypoint> Withdrawer {
    pub fn load(
        account: &'entrypoint mut Box<Account<'info, Self>>,
        programs_map: &'entrypoint ProgramsMap<'info>,
    ) -> Mutable<LoadedWithdrawer<'info, 'entrypoint>> {
        let owner = account.owner.clone();
        let last_withdraw = account.last_withdraw;

        Mutable::new(LoadedWithdrawer {
            __account__: account,
            __programs__: programs_map,
            owner,
            last_withdraw,
        })
    }

    pub fn store(loaded: Mutable<LoadedWithdrawer>) {
        let mut loaded = loaded.borrow_mut();
        let owner = loaded.owner.clone();

        loaded.__account__.owner = owner;

        let last_withdraw = loaded.last_withdraw;

        loaded.__account__.last_withdraw = last_withdraw;
    }
}

#[derive(Debug)]
pub struct LoadedWithdrawer<'info, 'entrypoint> {
    pub __account__: &'entrypoint mut Box<Account<'info, Withdrawer>>,
    pub __programs__: &'entrypoint ProgramsMap<'info>,
    pub owner: Pubkey,
    pub last_withdraw: i64,
}

pub fn deposit_handler<'info>(
    mut mint: SeahorseAccount<'info, '_, Mint>,
    mut signer_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut faucet_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut signer: SeahorseSigner<'info, '_>,
    mut n: u64,
) -> () {
    token::transfer(
        CpiContext::new(
            signer_account.programs.get("token_program"),
            token::Transfer {
                from: signer_account.to_account_info(),
                authority: signer.clone().to_account_info(),
                to: faucet_account.clone().to_account_info(),
            },
        ),
        <u64 as TryFrom<_>>::try_from(n.clone()).unwrap(),
    )
    .unwrap();
}

pub fn initialize_faucet_handler<'info>(
    mut mint: SeahorseAccount<'info, '_, Mint>,
    mut faucet: Empty<Mutable<LoadedFaucet<'info, '_>>>,
    mut signer: SeahorseSigner<'info, '_>,
    mut faucet_account: Empty<SeahorseAccount<'info, '_, TokenAccount>>,
    mut decimals: u64,
    mut max_withdraw: u64,
) -> () {
    let mut bump = faucet.bump.unwrap();
    let mut faucet = faucet.account.clone();

    faucet_account.account.clone();

    assign!(faucet.borrow_mut().bump, bump);

    assign!(faucet.borrow_mut().mint, mint.key());

    assign!(faucet.borrow_mut().decimals, decimals);

    assign!(faucet.borrow_mut().max_withdraw, max_withdraw);

    assign!(faucet.borrow_mut().owner, signer.key());
}

pub fn initialize_withdrawer_handler<'info>(
    mut signer: SeahorseSigner<'info, '_>,
    mut withdrawer: Empty<Mutable<LoadedWithdrawer<'info, '_>>>,
) -> () {
    let mut withdrawer = withdrawer.account.clone();

    assign!(withdrawer.borrow_mut().owner, signer.key());
}

pub fn withdraw_handler<'info>(
    mut mint: SeahorseAccount<'info, '_, Mint>,
    mut withdrawer_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut faucet_account: SeahorseAccount<'info, '_, TokenAccount>,
    mut faucet: Mutable<LoadedFaucet<'info, '_>>,
    mut n: u64,
    mut withdrawer: Mutable<LoadedWithdrawer<'info, '_>>,
    mut signer: SeahorseSigner<'info, '_>,
    mut clock: Sysvar<'info, Clock>,
) -> () {
    if !(mint.key() == faucet.borrow().mint) {
        panic!("The Token mint you are trying to withdraw does not match the faucet mint");
    }

    if !(signer.key() == withdrawer.borrow().owner) {
        panic!("You have provided a wrong Withdrawer account");
    }

    let mut timestamp = clock.unix_timestamp;

    if !((timestamp - 60) > withdrawer.borrow().last_withdraw) {
        panic!("Your transaction has been rate limited, please try again in one minute");
    }

    let mut bump = faucet.borrow().bump;
    let mut amount = n * faucet.borrow().decimals;

    if !(n <= faucet.borrow().max_withdraw) {
        panic!("The maximal amount you can withdraw is exeeded.");
    }

    token::transfer(
        CpiContext::new_with_signer(
            faucet_account.programs.get("token_program"),
            token::Transfer {
                from: faucet_account.to_account_info(),
                authority: faucet.borrow().__account__.to_account_info(),
                to: withdrawer_account.clone().to_account_info(),
            },
            &[Mutable::new(vec![
                "mint".to_string().as_bytes().as_ref(),
                mint.key().as_ref(),
                bump.to_le_bytes().as_ref(),
            ])
            .borrow()
            .as_slice()],
        ),
        amount.clone(),
    )
    .unwrap();

    assign!(withdrawer.borrow_mut().last_withdraw, timestamp);
}
