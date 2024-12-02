test:
	rg dead_code
	cargo pretty-test

peg:
	peginator-cli ./src/predikit/comp/pkparser.ebnf > ./src/predikit/comp/pkparser.rs
	rustfmt ./src/predikit/comp/pkparser.rs

