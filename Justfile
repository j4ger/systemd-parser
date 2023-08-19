expand:
	cd lib && cargo expand --example example

dump:
	cd syn/examples/dump-syntax && cargo run -- ../../../lib/examples/example.rs | bat

example:
	cargo run --example example	

dir:
	cargo run --example directory

patch:
	cargo run --example patch

dropin:
	cargo run --example dropins

