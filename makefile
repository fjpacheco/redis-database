VALGRIND = valgrind --leak-check=full --track-origins=yes --show-reachable=yes -s
 
run_server_database:
	cargo run --release --bin=server_database

run_server_html:
	cargo run --release --bin=server_html
	
run_server_html_valgrind:
	cargo b
	$(VALGRIND) target/debug/server

run_server_html_gdb:
	cargo b
	rust-gdb target/debug/server

unit_tests:
	cargo test

long_time_tests: 
	cargo test long_test -- --ignored

int_tests:
	cargo test int_test -- --test-threads 1 --ignored