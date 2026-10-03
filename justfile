_default:
	just --list

ci: fmt lint test doc no-std msrv test-ui miri

test:
    cargo test --all-features

test-ui:
    cargo +1.98.1 test --all-features --test ui -- --ignored

miri:
    cargo +nightly miri test --all-features

msrv: (check "1.56.0") (check "1.58.0") (check "1.83.0")

check VERSION:
    rm -f Cargo.lock
    cargo +{{VERSION}} check --all-features
    rm -f Cargo.lock

fmt:
    cargo fmt --check
    rustfmt --check --edition 2021 tests/ui/*.rs

lint:
    cargo clippy --all-targets --all-features -- -D warnings

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

no-std:
    cargo check --target thumbv6m-none-eabi --all-features
