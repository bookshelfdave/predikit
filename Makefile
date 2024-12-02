test:
	rg dead_code
	cargo pretty-test

install:
	cp ./target/debug/predikit /home/dparfitt/tools/bin/predikit
