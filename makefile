VALGRIND = valgrind --leak-check=full --track-origins=yes --show-reachable=yes -s
VALGRIND_2 = valgrind --tool=helgrind # Consular con Matías si es compatible con Rust ._.
 
run_v:
	cargo b
	$(VALGRIND) target/debug/server

run_v_heavy:
	cargo b
	$(VALGRIND_2) target/debug/server

run_gdb:
	cargo b
	rust-gdb target/debug/server
