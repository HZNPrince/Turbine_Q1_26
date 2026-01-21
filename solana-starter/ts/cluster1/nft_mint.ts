//Succesfully Minted! Check out your TX here:
//https://explorer.solana.com/tx/37yAS1A7VkNik8b64AmxCN9XKvChBjjK1HDBrdNpJ4DYsuM7hxtsuN5vw8Jia39u3SiNz8PwpUZUGF2SaqvSRTic?cluster=devnet
//Mint Address:  hztQGCXAJmjBxSg7xohwZ412jfAKRJQvqo4Z1AB5Jxc

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createSignerFromKeypair,
  signerIdentity,
  generateSigner,
  percentAmount,
} from "@metaplex-foundation/umi";
import {
  createNft,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";

import wallet from "../../wallet.json";
import base58 from "bs58";

const RPC_ENDPOINT = "https://api.devnet.solana.com";
const umi = createUmi(RPC_ENDPOINT);

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const myKeypairSigner = createSignerFromKeypair(umi, keypair);
umi.use(signerIdentity(myKeypairSigner));
umi.use(mplTokenMetadata());

const mint = generateSigner(umi);

(async () => {
  let tx = createNft(umi, {
    mint,
    name: "Earthy Rug",
    symbol: "ERug",
    uri: "https://gateway.irys.xyz/5iNPN7178ijBKFTkb5XtRXSqu5PKHBM9xNuo7C8tAwFy",
    sellerFeeBasisPoints: percentAmount(5),
  });
  let result = await tx.sendAndConfirm(umi);
  const signature = base58.encode(result.signature);

  console.log(
    `Succesfully Minted! Check out your TX here:\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`,
  );

  console.log("Mint Address: ", mint.publicKey);
})();
