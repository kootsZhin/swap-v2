import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SwapV2 } from "../target/types/swap_v2";

describe("swap-v2", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.SwapV2 as Program<SwapV2>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
