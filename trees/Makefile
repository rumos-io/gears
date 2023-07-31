benchmark:
	cargo bench

# Produces a table for comapring results against the go IAVL implementation
# Requires cargo-criterion, see https://github.com/bheisler/cargo-criterion
# Outputs results as JSON
benchmark-compare:
	cargo criterion --message-format=json > benchmark.json
	cargo run --example benchtable

profile:
	cargo bench --bench iavl_benchmark -- --profile-time=5

.PHONY: benchmark profile