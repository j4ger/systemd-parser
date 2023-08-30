expand:
	cd lib && cargo expand --example example

dump:
	cd syn/examples/dump-syntax && cargo run -- ../../../lib/examples/example.rs | bat

example:
	cargo run --example example	

template:
	cargo run --example template

dropin:
	cargo run --example dropins

