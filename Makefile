all:
	cargo build -p user_lib --release
	cargo build -p kernel --release

run:
	cargo run -p kernel --release

debug:
	RACHO_GDB=1 cargo run -p kernel --release

gdb_client:
	./tools/gdb_client.sh

disasm:
	cargo build -p kernel --release
	llvm-objdump -x target/riscv64gc-unknown-none-elf/release/kernel | less

disasm-vim:
	cargo build -p kernel --release
	llvm-objdump -x target/riscv64gc-unknown-none-elf/release/kernel > /tmp/racho.asm
	vim /tmp/racho.asm

clean:
	cargo clean
