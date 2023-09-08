expand:
	cd lib && cargo expand --example example

dump:
	cd syn/examples/dump-syntax && cargo run -- ../../../lib/examples/example.rs | bat

example:
	cargo run --example example	

template:
	cargo run --example template

dropins:
	cargo run --example dropins

subdir:
	cargo run --example subdir

specifiers:
	cargo run --example specifiers

