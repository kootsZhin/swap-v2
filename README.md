# Swap

Swap provides a convenient API to the Serum DEX for performing instantly settled token swaps directly on the orderbook.

## How it works

This is a modified version on [`project-serum/swap`](https://github.com/project-serum/swap/blob/master/README.md) with the use of [`SendTake`](https://github.com/project-serum/serum-dex/commit/4713d2f338d27b41ff09cf769e52d6a9f6c1cf4e) which does not require an open orders account or settlement of funds for the users - rather, it immediately deposits funds into the user account if there is a counterparty in the orderbook.

### `swap`

On `Token A / USD(x)` market,

```
Swap USD(x) -> Token A
```

or  

```
Swap Token A -> USD(x)
```

A convenience API to call the SendTake function on the Serum DEX.  

SendTake does not require an open orders account or settlement of funds for the user - rather it immediately settles the deposites funds into the user's account if there is a counterparty in the orderbook.  

Thus, this function is useful for instant swaps on a single A/B market, where A is the base currency and B is the quote currency.  

When side is "bid", then swaps B for A. When side is "ask", then swaps A for B.  

When side is 'bid', amount -> B, amount_out_min -> A, the implied price (of A) is amount/amount_out_min, e.g. if amount = 1000, amount_out_min = 100, then the implied price is 1000/100 = 10.  

Similarly, when side is 'ask', amount -> A, amount_out_min -> B, the implied price (of A) is amount_out_min/amount.  

### `swap_transitive`

On `Token A / USD(x)` market & `Token B / USD(x)` market,

```
Swap Token A -> USD(x) -> Token B
```

or  

```
Swap Token B -> USD(x) -> Token A
```

Swap two base currencies across two different markets.  

That is, suppose there are two markets, A/USD(x) and B/USD(x).  

Then swaps token A for token B via  

1. Selling A to USD(x) on A/USD(x) market using SendTake.  
2. Buying B using the proceed USD(x) on B/USD(x) market using SendTake.  

## Developing

This program requires building the Serum DEX from source, which is done using git submodules.

### Install Submodules

Pull the source

```bash
git submodule init
git submodule update
```

### Build the DEX

```bash
cd deps/serum-dex/dex/ && cargo build-bpf && cd ../../../
```

### Build

[Anchor](https://github.com/coral-xyz/anchor) is used for developoment, and it's recommended workflow is used here. To get started, see the [guide](https://book.anchor-lang.com/).


```bash
anchor build
```

or

```bash
anchor build --verifiable
```

The `--verifiable` flag should be used before deploying so that your build artifacts can be deterministically generated with docker.

### Test

To run the test you have to first spin up a loaclnet.

```bash
solana-test-validator --reset
```

Deploy the built DEX program

```bash
cd deps/serum-dex/dex/ && solana program deploy ./target/deploy/serum_dex.so -u http://localhost:8899
```

Run the tests in `tests`.

```bash
anchor test --skip-local-validator
```
