all:
	cd user && cargo build --release
	cd kernel && cargo build --release
run:
	./run_tcp_off.sh
