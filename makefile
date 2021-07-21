VALGRIND = valgrind --leak-check=full --track-origins=yes --show-reachable=yes -s
 
run_v:
	cargo b
	$(VALGRIND) target/debug/server

run_gdb:
	cargo b
	rust-gdb target/debug/server