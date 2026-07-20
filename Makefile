all:
	cargo build -p user_lib --release
	cargo build -p kernel --release

run:
	cargo run -p kernel --release

debug:
	RACHO_GDB=1 cargo run -p kernel --release

gdb_client:
	./tools/gdb_client.sh

clean:
	cargo clean
