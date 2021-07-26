VALGRIND = valgrind --leak-check=full --track-origins=yes --show-reachable=yes -s
 
run_v:
	cargo b
	$(VALGRIND) target/debug/server

run_gdb:
	cargo b
	rust-gdb target/debug/server

unit_tests:
	cargo test

long_time_tests: 
	cargo test long_test -- --ignored

int_tests:
	cargo test int_test -- --test-threads 1 --ignored