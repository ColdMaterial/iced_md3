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

check:
	just fmt-check
	just test
	just doc-no-all-features
	just doc
	just deny

loc:
	cloc src/
