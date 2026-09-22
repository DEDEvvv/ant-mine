# ANT Mine

Solana on-chain mining program for [antfm.fun](https://antfm.fun).

Program ID: `5cP76Ch3B8f3Pj6mnXC5V19RhZFhuZBQhjNansoQFKp9`

Token mint: `D7ahcwSv6GkcBpPJEKizUz8eJFoV4g2FWaUmZHxargan`

## What it does

- Instruction 0: initialize config
- Instruction 1: claim one free miner (500 max, one per wallet, kind chosen on chain)
- Instruction 2: claim mined tokens into the signer wallet

Hash rates are 1.0 to 2.0 H/s. One H/s mines 5555.555555 tokens per day.

## Security

Report vulnerabilities to `zoeefm@proton.me`. See `SECURITY.md`.

## Build

```
solana-verify build --library-name ant_mine --base-image solanafoundation/solana-verifiable-build:4.0.3
```
