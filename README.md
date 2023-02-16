# halo2-playground
Halo2 code snippets for testing.

## Compile and Run

```bash
# normal circuit
cargo run -p cab2 --bin cab2

# example of a circuit with inconsistent prover semantics
cargo run -p cab2 --bin cab2-0

# example of an underconstrained circuit
cargo run -p cab2 --bin cab2-1

cargo +nightly test -p halo2-rsa

cargo run -p darkfi-0 --bin darkfi-0

cargo run -p snark-0 --bin snark-0
```

