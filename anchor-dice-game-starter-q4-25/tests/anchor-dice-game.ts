import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { AnchorDiceGame } from "../target/types/anchor_dice_game"
import {
  Ed25519Program,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Transaction,
} from "@solana/web3.js"
import { readFileSync } from "fs"
import assert from "assert"

describe("anchor-dice-game", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const houseWallet = provider.wallet.payer
  const connection = provider.connection

  const seed = new anchor.BN(Math.floor(Math.random() * 100000))
  const seeds = seed.toArrayLike(Buffer, "le", 16)
  const player = Keypair.generate()

  let vaultPda: PublicKey
  let betAccount: PublicKey
  before("Accounts and Keys setup", async () => {
    const signature = await connection.requestAirdrop(player.publicKey, 10000_000_000_000)
    await connection.confirmTransaction(signature)
    ;[vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), houseWallet.publicKey.toBuffer()],
      program.programId,
    )
    ;[betAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("bet"), vaultPda.toBuffer(), seeds],
      program.programId,
    )
  })

  const program = anchor.workspace.anchorDiceGame as Program<AnchorDiceGame>

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(new anchor.BN(1_000_000_00))
      .accounts({
        house: houseWallet.publicKey,
      })
      .rpc()
    console.log("Your transaction signature", tx)
  })

  it("Place Bet", async () => {
    const tx = await program.methods
      .placeBet(seed, 10, new anchor.BN(1000_000_000_000))
      .accounts({
        house: houseWallet.publicKey,
        player: player.publicKey,
        vault: vaultPda,
        bet: betAccount,
      })
      .signers([player])
      .rpc()

    console.log("Placed bet completed: ", tx)
  })

  it("Resolve bet", async () => {
    let account = await connection.getAccountInfo(betAccount, "processed")
    if (!account) {
      throw new Error(`Bet account ${betAccount.toBase58()} not found`)
    }

    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: houseWallet.secretKey,
      message: account.data.subarray(8),
    })

    const resolve_sig = await program.methods
      .resolveBet(Buffer.from(sig_ix.data.subarray(16 + 32, 16 + 32 + 64)))
      .accounts({
        house: houseWallet.publicKey,
        player: player.publicKey,
        bet: betAccount,
        instructionSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .signers([houseWallet])
      .instruction()

    console.log("Ed25519 Program ID:", sig_ix.programId.toBase58())
    const tx = new Transaction().add(sig_ix).add(resolve_sig)
    const sig = await sendAndConfirmTransaction(connection, tx, [houseWallet])

    console.log("Bet resolved successfully: ", sig)
  })

  it("Refund bet after resolving : Should fail", async () => {
    try {
      const tx = await program.methods
        .refundBet()
        .accounts({
          player: player.publicKey,
          house: houseWallet.publicKey,
        })
        .rpc()

      assert.fail("Expected transaction to fail")
    } catch (_e) {
      console.error("Transaction correctly rejected .")
    }
  })

  it("Place a bet and refund correctly", async () => {
    const tx1 = await program.methods
      .placeBet(seed, 50, new anchor.BN(1000_000_000_000))
      .accounts({
        house: houseWallet.publicKey,
        player: player.publicKey,
        vault: vaultPda,
        bet: betAccount,
      })
      .signers([player])
      .rpc()

    console.log("Placed bet completed: ", tx1)

    const tx2 = await program.methods
      .refundBet()
      .accounts({
        player: player.publicKey,
        house: houseWallet.publicKey,
        bet: betAccount,
      })
      .signers([player])
      .rpc()

    console.log("Bet refunded : ", tx2)
  })
})
