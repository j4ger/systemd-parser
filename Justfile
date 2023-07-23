expand:
	cd lib && cargo expand --example example

dump:
	cd syn/examples/dump-syntax && cargo run -- ../../../lib/examples/example.rs | bat

example:
	cd lib && cargo run --example example	

