CRUNTIME ?= docker
KERNEL_TAG ?=
EXTRA_DOCKER_FLAGS ?=

ifneq ($(KERNEL_TAG),)
EXTRA_DOCKER_FLAGS += --build-arg KERNEL_TAG=$(KERNEL_TAG)
endif

.PHONY: all
all: build fmt lint

.PHONY: build
build:
	$(CRUNTIME) build \
		--target artifact \
		--output type=local,dest=out \
		$(EXTRA_DOCKER_FLAGS) \
		.

.PHONY: fmt
fmt:
	$(CRUNTIME) build \
		--target rustfmt \
		--tag enclave-rustfmt \
		$(EXTRA_DOCKER_FLAGS) \
		.
	$(CRUNTIME) run \
		--rm -i \
		-v $(shell pwd):/usr/local/src/enclave \
		enclave-rustfmt

.PHONY: lint
lint:
	$(CRUNTIME) build \
		--target clippy \
		--tag enclave-clippy \
		$(EXTRA_DOCKER_FLAGS) \
		.
	$(CRUNTIME) run \
		--rm -i \
		-v $(shell pwd):/usr/local/src/enclave \
		enclave-clippy
