expand:
	cd macro && cargo expand --example example

dump:
	cd syn/examples/dump-syntax && cargo run -- ../../../macro/examples/example.rs | bat
	
