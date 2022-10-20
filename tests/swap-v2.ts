import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Coin, Dex, DexMarket, FileKeypair } from "@project-serum/serum-dev-tools";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { SwapV2 } from "../target/types/swap_v2";

const DEX_ADDRESS = '9cnJvRQY38Bu7dWUUCncZ53evxZ4mR4S9vYV8BpToh26';

describe("swap-v2", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(
    anchor.AnchorProvider.local('http://localhost:8899', {
      preflightCommitment: "confirmed",
      commitment: "confirmed",
    })
  );

  // anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.SwapV2 as Program<SwapV2>;
  const connection = program.provider.connection;

  const dexAddres = new PublicKey(DEX_ADDRESS);

  let BTC: Coin, ETH: Coin, USDC: Coin,
    btcMarket: DexMarket,
    ethMarket: DexMarket,
    btcMarketVaultSigner: PublicKey,
    ethMarketVaultSigner: PublicKey;


  const MarketsOwner = FileKeypair.loadOrGenerate("./owner.json");
  const marketsOwner = MarketsOwner.keypair;

  const Alice = Keypair.generate();

  let
    aliceBtcAccount: PublicKey,
    aliceEthAccount: PublicKey,
    aliceUsdcAccount: PublicKey;

  const dex = new Dex(dexAddres, connection);

  it("BOILERPLATE: Sets up the dex, coins and the markets", async () => {
    await connection.confirmTransaction(
      await connection.requestAirdrop(
        marketsOwner.publicKey,
        10 * LAMPORTS_PER_SOL
      ),
      'confirmed'
    );

    BTC = await dex.createCoin('BTC', 9, marketsOwner, marketsOwner, marketsOwner);
    ETH = await dex.createCoin('ETH', 9, marketsOwner, marketsOwner, marketsOwner);
    USDC = await dex.createCoin('USDC', 9, marketsOwner, marketsOwner, marketsOwner);

    btcMarket = await dex.initDexMarket(marketsOwner, BTC, USDC, {
      lotSize: 1e-3,
      tickSize: 1e-2,
    });

    [btcMarketVaultSigner] = await PublicKey.findProgramAddress(
      [btcMarket.address.toBuffer()],
      program.programId
    );

    console.log(`Created ${btcMarket.marketSymbol} market @ ${btcMarket.address.toString()}`);

    ethMarket = await dex.initDexMarket(marketsOwner, ETH, USDC, {
      lotSize: 1e-3,
      tickSize: 1e-2,
    });

    [ethMarketVaultSigner] = await PublicKey.findProgramAddress(
      [ethMarket.address.toBuffer()],
      program.programId
    );

    console.log(`Created ${ethMarket.marketSymbol} market @ ${ethMarket.address.toString()}`);

    await BTC.fundAccount(100, marketsOwner, connection);
    await ETH.fundAccount(1000000, marketsOwner, connection);
    await USDC.fundAccount(100000000, marketsOwner, connection);

    dex.runMarketMaker(btcMarket, MarketsOwner, {
      durationInSecs: 30,
      orderCount: 3,
      initialBidSize: 1000,
      baseGeckoSymbol: "bitcoin",
      quoteGeckoSymbol: "usd",
      verbose: true,
    });

    dex.runMarketMaker(ethMarket, MarketsOwner, {
      durationInSecs: 30,
      orderCount: 3,
      initialBidSize: 1000,
      baseGeckoSymbol: "ethereum",
      quoteGeckoSymbol: "usd",
      verbose: true,
    });

    dex.runCrank(btcMarket, MarketsOwner, {
      durationInSecs: 20,
      verbose: true,
    });

    dex.runCrank(ethMarket, MarketsOwner, {
      durationInSecs: 20,
      verbose: true,
    });
  });

  it("BOILERPLATE: Sets up account for Alice", async () => {
    await connection.confirmTransaction(
      await connection.requestAirdrop(
        Alice.publicKey,
        10 * LAMPORTS_PER_SOL
      ),
      'confirmed'
    );

    await BTC.fundAccount(100, Alice, connection);
    await ETH.fundAccount(1000000, Alice, connection);
    await USDC.fundAccount(100000000, Alice, connection);

    aliceBtcAccount = await getAssociatedTokenAddress(BTC.mint, Alice.publicKey);
    aliceEthAccount = await getAssociatedTokenAddress(ETH.mint, Alice.publicKey);
    aliceUsdcAccount = await getAssociatedTokenAddress(USDC.mint, Alice.publicKey);
  });

  it('Swap from BTC -> USDC', async () => {
    const tx = await program.methods.test().transaction();
    const txid = await program.provider.sendAndConfirm(tx, [], { skipPreflight: true });
    console.log(txid);

    // await program.methods
    //   .swap(
    //     Side.Ask,
    //     new anchor.BN(100),
    //     new anchor.BN(0),
    //   )
    //   .accounts({
    //     market: {
    //       market: btcMarket.address,
    //       requestQueue: btcMarket.serumMarket.decoded.requestQueue,
    //       eventQueue: btcMarket.serumMarket.decoded.eventQueue,
    //       marketBids: btcMarket.serumMarket.decoded.bids,
    //       marketAsks: btcMarket.serumMarket.decoded.asks,
    //       coinVault: btcMarket.serumMarket.decoded.baseVault,
    //       pcVault: btcMarket.serumMarket.decoded.quoteVault,
    //       vaultSigner: btcMarketVaultSigner,
    //       coinWallet: aliceBtcAccount,
    //     },
    //     walletOwner: Alice.publicKey,
    //     pcWallet: aliceUsdcAccount,
    //     dexProgram: dexAddres,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //   })
    //   .signers([Alice])
    // .rpc();
  })
});

// Side rust enum used for the program's RPC API.
const Side = {
  Bid: { bid: {} },
  Ask: { ask: {} },
};