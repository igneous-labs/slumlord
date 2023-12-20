# slumlord

Simple zero-fee SOL flash loan program for paying accounts rent.

## Usage

- `Borrow` transfers `slumlord_balance - 1` lamports from `slumlord` account to specified `dst` account.
  - Can be called from CPI
- `CheckRepaid` instruction must be a top-level instruction of the transaction and follow the `Borrow` instruction
  - User must make sure to return at least the same amount of `slumlord_balance - 1` to `slumlord` account before calling `CheckRepaid`

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

```sh
solana-verify get-executable-hash target/deploy/slumlord.so
```

```sh
solana-verify get-program-hash s1umBj7CEUA6djs6V1c6o2Nym3QrqF4ryKDr1Nm1FKt
```

Hash:

```sh
c81aa8bdbc8d380c53f7ca2942581f6c15e250c5f1b195822256e60e765c19d9
```
