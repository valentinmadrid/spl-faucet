import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Faucet } from "../target/types/faucet";

describe("faucet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const USDC_MINT_ADDRESS = new anchor.web3.PublicKey(
    "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
  );

  const faucet = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint"), USDC_MINT_ADDRESS.toBuffer()],
    anchor.workspace.Faucet.programId
  )[0];

  const faucet_token_account = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("token-seed"), USDC_MINT_ADDRESS.toBuffer()],
    anchor.workspace.Faucet.programId
  )[0];

  const program = anchor.workspace.Faucet as Program<Faucet>;

  it("Initialize Faucet", async () => {
    // Add your test here.
    const tx = await program.methods
      .initializeFaucet(new anchor.BN(1000000), new anchor.BN(10000))
      .accounts({
        mint: USDC_MINT_ADDRESS,
        faucet: faucet,
        faucetAccount: faucet_token_account,
        signer: anchor.workspace.Faucet.provider.wallet.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });
});
