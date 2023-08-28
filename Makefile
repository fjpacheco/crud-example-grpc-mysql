.PHONY: up cargo-test down command

test: up cargo-test down

up:
	docker-compose up --build -d

cargo-test:
	cargo test

down:
	docker-compose down