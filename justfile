doc-no-all-features:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p iced_m3

doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p iced_m3 --all-features

open-doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --no-deps -p iced_m3 --all-features --open

fmt:
	cargo fmt --workspace

fmt-check:
	cargo fmt --check --all

test:
	cargo test --workspace
	cargo test --workspace --no-default-features

deny:
	cargo deny check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cargo clippy --workspace --all-targets --no-default-features -- -D warnings

check:
	just fmt-check
	just clippy
	just test
	just doc-no-all-features
	just doc
	just deny

loc:
	cloc src/
