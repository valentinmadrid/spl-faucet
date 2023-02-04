from seahorse.prelude import *

# This is your program's public key and it will update
# automatically when you build the project.
declare_id('GCGsUSYteThHoeZeT78AXHjLKmwVCVsP8WqT3GhSMLua')

class Faucet(Account):
  bump: u8
  mint: Pubkey
  owner: Pubkey
  max_withdraw: u64
  decimals: u64

class Withdrawer(Account):
  owner: Pubkey
  last_withdraw: i64

@instruction
def initialize_faucet(mint: TokenMint, faucet: Empty[Faucet], signer: Signer, faucet_account: Empty[TokenAccount], decimals: u64, max_withdraw: u64):
  bump = faucet.bump()
  faucet = faucet.init(
    payer = signer,
    seeds = ['mint', mint]
  )
  faucet_account.init(
    payer = signer,
    seeds = ["token-seed", mint],
    mint = mint,
    authority = faucet,
  )
  faucet.bump = bump
  faucet.mint = mint.key()
  faucet.decimals = decimals
  faucet.max_withdraw = max_withdraw
  faucet.owner = signer.key()

@instruction
def deposit(mint: TokenMint, signer_account: TokenAccount, faucet_account: TokenAccount, signer: Signer, n: u64):
  signer_account.transfer(
    authority = signer,
    to = faucet_account,
    amount = u64(n)
  )

@instruction
def initialize_withdrawer(signer: Signer, withdrawer: Empty[Withdrawer]):
  withdrawer = withdrawer.init(
    payer = signer,
    seeds = ['withdrawer', signer]
  )
  withdrawer.owner = signer.key()

@instruction
def withdraw(mint: TokenMint, 
             withdrawer_account: TokenAccount, 
             faucet_account: TokenAccount, 
             faucet: Faucet,
             n: u64, 
             withdrawer: Withdrawer,
             signer: Signer,
             clock: Clock
            ):
  assert mint.key() == faucet.mint, 'The Token mint you are trying to withdraw does not match the faucet mint'
  assert signer.key() == withdrawer.owner, 'You have provided a wrong Withdrawer account'
  timestamp:  i64 = clock.unix_timestamp()            
  assert timestamp - 60 > withdrawer.last_withdraw, 'Your transaction has been rate limited, please try again in one minute'
  bump = faucet.bump
  amount = n * faucet.decimals
  assert n <= faucet.max_withdraw, 'The maximal amount you can withdraw is exeeded.'
  faucet_account.transfer(
    authority = faucet,
    to = withdrawer_account,
    amount = amount,
    signer = ['mint', mint, bump]
  )
  withdrawer.last_withdraw = timestamp
