import wallet from "../../TurHCThfhhm41FptMhG7C8daUBLaEjVQJnMZTybj9yi.json"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults"
import {
  createMetadataAccountV3,
  CreateMetadataAccountV3InstructionAccounts,
  CreateMetadataAccountV3InstructionArgs,
  DataV2Args,
} from "@metaplex-foundation/mpl-token-metadata"
import { createSignerFromKeypair, signerIdentity, publicKey } from "@metaplex-foundation/umi"
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes"

// Define our Mint address
const mint = publicKey("5hPgadD2pFvuehEaH91NXgRpmZpdghTgBcSeragdXSQq")

// Create a UMI connection
const umi = createUmi("https://api.devnet.solana.com")
const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet))
const signer = createSignerFromKeypair(umi, keypair)
umi.use(signerIdentity(createSignerFromKeypair(umi, keypair)))

;(async () => {
  try {
    // Start here
    let accounts: CreateMetadataAccountV3InstructionAccounts = { mint, mintAuthority: signer }

    let data: DataV2Args = {
      name: "CringeToken",
      symbol: "Crngy",
      uri: "http://example.com/uri",
      sellerFeeBasisPoints: 500,
      creators: null,
      collection: null,
      uses: null,
    }

    let args: CreateMetadataAccountV3InstructionArgs = {
      data: data,
      isMutable: true,
      collectionDetails: null,
    }

    let tx = createMetadataAccountV3(umi, {
      ...accounts,
      ...args,
    })

    let result = await tx.sendAndConfirm(umi)
    console.log(bs58.encode(result.signature))
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`)
  }
})()
