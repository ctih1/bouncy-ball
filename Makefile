wasm src/main.rs:
	cargo build --release --target wasm32-unknown-unknown

windows src/main.rs:
	cargo build --release --target x86_64-pc-windows-msvc

linux src/main.rs:
	cargo build --release --target x86_64-unknown-linux-gnu
