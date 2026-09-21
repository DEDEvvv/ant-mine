# ANT Mine

Solana on-chain mining program for [antfm.fun](https://antfm.fun).

Program ID: `APNv3yTr5Zd5UYrv2LpHnfoLQ1WKA8K3gZ1Sk3DZeZL5`

Token mint: `7PrUyJot9dKnuycunNwBLQQS84fiqTPE7XcfWdtYcgan`

## What it does

- Instruction 0: initialize vault config
- Instruction 1: buy a miner (pays SOL to treasury, creates a miner PDA)
- Instruction 2: claim rewards from the program token vault

Catalog prices and hash rates are hardcoded in `src/lib.rs`. Starter miner (`kind = 1`) can be claimed once per wallet.

## Security

Report vulnerabilities to `zoeefm@proton.me`. See `SECURITY.md`.

## Build

```
cargo build-sbf
```
