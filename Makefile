CRUNTIME ?= docker

.PHONY: all
all: build fmt lint

.PHONY: build
build:
	$(CRUNTIME) build --target artifact --output type=local,dest=out .

.PHONY: fmt
fmt:
	$(CRUNTIME) build --target rustfmt --tag enclave-rustfmt .
	$(CRUNTIME) run --rm -i -v $(shell pwd):/usr/local/src/enclave enclave-rustfmt

.PHONY: lint
lint:
	$(CRUNTIME) build --target clippy --tag enclave-clippy .
	$(CRUNTIME) run --rm -i -v $(shell pwd):/usr/local/src/enclave enclave-clippy
