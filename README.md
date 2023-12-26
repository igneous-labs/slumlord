# slumlord

Simple zero-fee SOL flash loan program for paying accounts rent.

## Usage

- `Borrow` transfers `slumlord_balance - 1` lamports from `slumlord` account to specified `dst` account.
  - Can be called from CPI
- `CheckRepaid` instruction must be a top-level instruction of the transaction and follow the `Borrow` instruction
  - User must make sure to return at least the same amount of `slumlord_balance - 1` to `slumlord` account before calling `CheckRepaid`
  - Idempotent, can be called from CPI. If no flash loan is active, this will just be a successful no-op
- `Repay` instruction transfers the outstanding loan balance from the specified SystemAccount to `slumlord`
  - Allows users to easily repay the flash loan without having to read the loan amount from the `slumlord` account.

If you're composing with slumlord via CPI in your own program, consider making use of `CheckRepaid`'s idempotency and calling it in your program to end the loan where appropriate. This allows your program to be composed with subsequent `Borrow`s while still only requiring a single top-level `CheckRepaid` instruction at the end.

## Setup

Match solana + rust toolchain versions of `ellipsislabs/solana:1.16.20` to ensure build close to reproducible build as possible.

```sh
sh -c "$(curl -sSfL https://release.solana.com/v1.16.20/install)"
cargo-build-sbf --version && rustc --version

solana-cargo-build-sbf 1.16.20
platform-tools v1.37
rustc 1.68.0 (2c8cc3432 2023-03-06)
```

## Interface Generation

`idl.json` is a handwritten shank style IDL from which the `slumlord_interface` crate is generated.

To regenerate the `slumlord_interface` crate, run in workspace root:

```sh
solores \
    -z Slumlord \
    --solana-program-vers "workspace=true" \
    --borsh-vers "workspace=true" \
    --thiserror-vers "workspace=true" \
    --num-derive-vers "workspace=true" \
    --num-traits-vers "workspace=true" \
    --serde-vers "workspace=true" \
    --bytemuck-vers "workspace=true" \
    idl.json
```

Crate generated with solores 0.5.0

## Verifiable build

Using [ellipsislabs/solana:1.16.20](https://github.com/Ellipsis-Labs/solana-verifiable-build/blob/master/docker/v1.16.20.Dockerfile)

### Build

In workspace root:

```sh
solana-verify build
```

### Verify

In workspace root after build:

```sh
solana-verify get-executable-hash target/deploy/slumlord.so
```

Against mainnet deploy:

```sh
solana-verify get-program-hash s1umBj7CEUA6djs6V1c6o2Nym3QrqF4ryKDr1Nm1FKt
```

### Hash

```sh
17d20483ee24bb0c1d0fead460f8eee7ccfc9bfcd9c811d295d3a56cc8e96065
```
