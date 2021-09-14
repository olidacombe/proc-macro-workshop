export RUSTFLAGS = -Z macro-backtrace
export RUST_BACKTRACE = 1

test-%:
	pushd $*; cargo test; popd
