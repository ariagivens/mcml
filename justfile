test:
    cargo run -- tests/literals.mcml -o /tmp/literals
    cd ../mctest && cargo run --release -- /tmp/literals