import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";

describe("vault", () => {
  // Set up our connection to Solana
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Vault as Program<Vault>;

  // Figure out where our vault state account will live (using the user's wallet as seed)
  const vaultState = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), provider.wallet.publicKey.toBuffer()],
    program.programId
  )[0];

  // Figure out where the actual SOL will be stored (using vault state as seed)
  const vault = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultState.toBuffer()],
    program.programId
  )[0];

  it("Should initialize vault", async () => {
    // Create a new vault for the user
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        signer: provider.wallet.publicKey, // Who's creating the vault
        vaultState: vaultState, // Where we store vault info
        vault: vault, // Where the SOL will be kept
        systemProgram: anchor.web3.SystemProgram.programId, // Needed to move SOL around
      })
      .rpc();
    console.log("Initialize tx:", tx);
  });

  it("Should deposit SOL", async () => {
    // Put some SOL into the vault
    const depositAmount = new anchor.BN(10000000); // 0.01 SOL (in lamports)
    const tx = await program.methods
      .deposit(depositAmount)
      .accountsPartial({
        signer: provider.wallet.publicKey, // Who's putting money in
        vaultState: vaultState, // Vault info
        vault: vault, // Where the money goes
        systemProgram: anchor.web3.SystemProgram.programId, // To transfer the SOL
      })
      .rpc();
    console.log("Deposit tx:", tx);
  });

  it("Should withdraw SOL", async () => {
    // Take some SOL out of the vault
    const withdrawAmount = new anchor.BN(5000000); // 0.005 SOL (in lamports)
    const tx = await program.methods
      .withdraw(withdrawAmount)
      .accountsPartial({
        signer: provider.wallet.publicKey, // Who's taking money out
        vaultState: vaultState, // Vault info
        vault: vault, // Where the money comes from
        systemProgram: anchor.web3.SystemProgram.programId, // To transfer the SOL
      })
      .rpc();
    console.log("Withdraw tx:", tx);
  });

  it("Should close vault", async () => {
    // Shut down the vault and get all money back
    const tx = await program.methods
      .close()
      .accountsPartial({
        signer: provider.wallet.publicKey, // Who's closing the vault
        vaultState: vaultState, // Vault info (will be deleted)
        vault: vault, // Where the money is (will be emptied)
        systemProgram: anchor.web3.SystemProgram.programId, // To transfer remaining SOL
      })
      .rpc();
    console.log("Close tx:", tx);
  });
});
