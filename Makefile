develop:
	cargo watch -c -x 'check' -x 'clippy -- -D warnings' -x 'test'

develop-linux:
	docker build -f tests/Dockerfile -t diffmetrik-test .
	docker run -it -v $(shell pwd):/app diffmetrik-test 

show-outdated:
	cargo outdated
