#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

pub mod dot;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token, TokenAccount},
};

use dot::program::*;
use std::{cell::RefCell, rc::Rc};

declare_id!("EtTeTRSJSRBBgm5nrmodadBpToGFrwWjo2syiVAjvjuT");

pub mod seahorse_util {
    use super::*;

    #[cfg(feature = "pyth-sdk-solana")]
    pub use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};
    use std::{collections::HashMap, fmt::Debug, ops::Deref};

    pub struct Mutable<T>(Rc<RefCell<T>>);

    impl<T> Mutable<T> {
        pub fn new(obj: T) -> Self {
            Self(Rc::new(RefCell::new(obj)))
        }
    }

    impl<T> Clone for Mutable<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Deref for Mutable<T> {
        type Target = Rc<RefCell<T>>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T: Debug> Debug for Mutable<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }

    impl<T: Default> Default for Mutable<T> {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T: Clone> Mutable<Vec<T>> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    impl<T: Clone, const N: usize> Mutable<[T; N]> {
        pub fn wrapped_index(&self, mut index: i128) -> usize {
            if index >= 0 {
                return index.try_into().unwrap();
            }

            index += self.borrow().len() as i128;

            return index.try_into().unwrap();
        }
    }

    #[derive(Clone)]
    pub struct Empty<T: Clone> {
        pub account: T,
        pub bump: Option<u8>,
    }

    #[derive(Clone, Debug)]
    pub struct ProgramsMap<'info>(pub HashMap<&'static str, AccountInfo<'info>>);

    impl<'info> ProgramsMap<'info> {
        pub fn get(&self, name: &'static str) -> AccountInfo<'info> {
            self.0.get(name).unwrap().clone()
        }
    }

    #[derive(Clone, Debug)]
    pub struct WithPrograms<'info, 'entrypoint, A> {
        pub account: &'entrypoint A,
        pub programs: &'entrypoint ProgramsMap<'info>,
    }

    impl<'info, 'entrypoint, A> Deref for WithPrograms<'info, 'entrypoint, A> {
        type Target = A;

        fn deref(&self) -> &Self::Target {
            &self.account
        }
    }

    pub type SeahorseAccount<'info, 'entrypoint, A> =
        WithPrograms<'info, 'entrypoint, Box<Account<'info, A>>>;

    pub type SeahorseSigner<'info, 'entrypoint> = WithPrograms<'info, 'entrypoint, Signer<'info>>;

    #[derive(Clone, Debug)]
    pub struct CpiAccount<'info> {
        #[doc = "CHECK: CpiAccounts temporarily store AccountInfos."]
        pub account_info: AccountInfo<'info>,
        pub is_writable: bool,
        pub is_signer: bool,
        pub seeds: Option<Vec<Vec<u8>>>,
    }

    #[macro_export]
    macro_rules! seahorse_const {
        ($ name : ident , $ value : expr) => {
            macro_rules! $name {
                () => {
                    $value
                };
            }

            pub(crate) use $name;
        };
    }

    #[macro_export]
    macro_rules! assign {
        ($ lval : expr , $ rval : expr) => {{
            let temp = $rval;

            $lval = temp;
        }};
    }

    #[macro_export]
    macro_rules! index_assign {
        ($ lval : expr , $ idx : expr , $ rval : expr) => {
            let temp_rval = $rval;
            let temp_idx = $idx;

            $lval[temp_idx] = temp_rval;
        };
    }

    pub(crate) use assign;

    pub(crate) use index_assign;

    pub(crate) use seahorse_const;
}

#[program]
mod faucet {
    use super::*;
    use seahorse_util::*;
    use std::collections::HashMap;

    #[derive(Accounts)]
    # [instruction (n : u64)]
    pub struct Deposit<'info> {
        #[account(mut)]
        pub mint: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub signer_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub faucet_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub signer: Signer<'info>,
        pub token_program: Program<'info, Token>,
    }

    pub fn deposit(ctx: Context<Deposit>, n: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let mint = SeahorseAccount {
            account: &ctx.accounts.mint,
            programs: &programs_map,
        };

        let signer_account = SeahorseAccount {
            account: &ctx.accounts.signer_account,
            programs: &programs_map,
        };

        let faucet_account = SeahorseAccount {
            account: &ctx.accounts.faucet_account,
            programs: &programs_map,
        };

        let signer = SeahorseSigner {
            account: &ctx.accounts.signer,
            programs: &programs_map,
        };

        deposit_handler(
            mint.clone(),
            signer_account.clone(),
            faucet_account.clone(),
            signer.clone(),
            n,
        );

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (decimals : u64 , max_withdraw : u64)]
    pub struct InitializeFaucet<'info> {
        #[account(mut)]
        pub mint: Box<Account<'info, Mint>>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Faucet > () + 8 , payer = signer , seeds = ["mint" . as_bytes () . as_ref () , mint . key () . as_ref ()] , bump)]
        pub faucet: Box<Account<'info, dot::program::Faucet>>,
        #[account(mut)]
        pub signer: Signer<'info>,
        # [account (init , payer = signer , seeds = ["token-seed" . as_bytes () . as_ref () , mint . key () . as_ref ()] , bump , token :: mint = mint , token :: authority = faucet)]
        pub faucet_account: Box<Account<'info, TokenAccount>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }

    pub fn initialize_faucet(
        ctx: Context<InitializeFaucet>,
        decimals: u64,
        max_withdraw: u64,
    ) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let mint = SeahorseAccount {
            account: &ctx.accounts.mint,
            programs: &programs_map,
        };

        let faucet = Empty {
            account: dot::program::Faucet::load(&mut ctx.accounts.faucet, &programs_map),
            bump: ctx.bumps.get("faucet").map(|bump| *bump),
        };

        let signer = SeahorseSigner {
            account: &ctx.accounts.signer,
            programs: &programs_map,
        };

        let faucet_account = Empty {
            account: SeahorseAccount {
                account: &ctx.accounts.faucet_account,
                programs: &programs_map,
            },
            bump: ctx.bumps.get("faucet_account").map(|bump| *bump),
        };

        initialize_faucet_handler(
            mint.clone(),
            faucet.clone(),
            signer.clone(),
            faucet_account.clone(),
            decimals,
            max_withdraw,
        );

        dot::program::Faucet::store(faucet.account);

        return Ok(());
    }

    #[derive(Accounts)]
    pub struct InitializeWithdrawer<'info> {
        #[account(mut)]
        pub signer: Signer<'info>,
        # [account (init , space = std :: mem :: size_of :: < dot :: program :: Withdrawer > () + 8 , payer = signer , seeds = ["withdrawer" . as_bytes () . as_ref () , signer . key () . as_ref ()] , bump)]
        pub withdrawer: Box<Account<'info, dot::program::Withdrawer>>,
        pub rent: Sysvar<'info, Rent>,
        pub system_program: Program<'info, System>,
    }

    pub fn initialize_withdrawer(ctx: Context<InitializeWithdrawer>) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "system_program",
            ctx.accounts.system_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let signer = SeahorseSigner {
            account: &ctx.accounts.signer,
            programs: &programs_map,
        };

        let withdrawer = Empty {
            account: dot::program::Withdrawer::load(&mut ctx.accounts.withdrawer, &programs_map),
            bump: ctx.bumps.get("withdrawer").map(|bump| *bump),
        };

        initialize_withdrawer_handler(signer.clone(), withdrawer.clone());

        dot::program::Withdrawer::store(withdrawer.account);

        return Ok(());
    }

    #[derive(Accounts)]
    # [instruction (n : u64)]
    pub struct Withdraw<'info> {
        #[account(mut)]
        pub mint: Box<Account<'info, Mint>>,
        #[account(mut)]
        pub withdrawer_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub faucet_account: Box<Account<'info, TokenAccount>>,
        #[account(mut)]
        pub faucet: Box<Account<'info, dot::program::Faucet>>,
        #[account(mut)]
        pub withdrawer: Box<Account<'info, dot::program::Withdrawer>>,
        #[account(mut)]
        pub signer: Signer<'info>,
        #[account()]
        pub clock: Sysvar<'info, Clock>,
        pub token_program: Program<'info, Token>,
    }

    pub fn withdraw(ctx: Context<Withdraw>, n: u64) -> Result<()> {
        let mut programs = HashMap::new();

        programs.insert(
            "token_program",
            ctx.accounts.token_program.to_account_info(),
        );

        let programs_map = ProgramsMap(programs);
        let mint = SeahorseAccount {
            account: &ctx.accounts.mint,
            programs: &programs_map,
        };

        let withdrawer_account = SeahorseAccount {
            account: &ctx.accounts.withdrawer_account,
            programs: &programs_map,
        };

        let faucet_account = SeahorseAccount {
            account: &ctx.accounts.faucet_account,
            programs: &programs_map,
        };

        let faucet = dot::program::Faucet::load(&mut ctx.accounts.faucet, &programs_map);
        let withdrawer =
            dot::program::Withdrawer::load(&mut ctx.accounts.withdrawer, &programs_map);

        let signer = SeahorseSigner {
            account: &ctx.accounts.signer,
            programs: &programs_map,
        };

        let clock = &ctx.accounts.clock.clone();

        withdraw_handler(
            mint.clone(),
            withdrawer_account.clone(),
            faucet_account.clone(),
            faucet.clone(),
            n,
            withdrawer.clone(),
            signer.clone(),
            clock.clone(),
        );

        dot::program::Faucet::store(faucet);

        dot::program::Withdrawer::store(withdrawer);

        return Ok(());
    }
}
