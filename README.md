# Uniswap Arbitrage Simulator 

To run the unit tests, run the following command
```shell
cargo test
```

To run an integration test, run the following command
```shell
cargo run
```

## Design

Pools are designed as observables and notifies all listeners. This is done to simulate a bot listening for transactions in the Ethereum mempool. Whenever a swap is carried out, the `run_arbitrage` function runs and tries to arbitrage the price difference.

## Arbitrage Logic

To derive the formula, the following constraints were used:

1. The change in `DAI` in pool 1 and pool 2 must be the same
1. `k` remains the same after a swap in each pool (almost)
1. The ratio of tokens in each must be the same after the swap. If not, then more arbitrage opportunities can be found after the initial one
