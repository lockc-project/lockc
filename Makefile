BPFTOOL ?= bpftool
CARGO ?= cargo
CLANG ?= clang
CRUNTIME ?= docker
CURL ?= curl
KERNEL_TAG ?=
EXTRA_DOCKER_FLAGS ?=
CONTAINERIZED_BUILD ?= 1

PREFIX ?= $(DESTDIR)/usr/local
BINDIR ?= $(PREFIX)/bin
UNITDIR ?= $(PREFIX)/lib/systemd/system

ifneq ($(KERNEL_TAG),)
EXTRA_DOCKER_FLAGS += --build-arg KERNEL_TAG=$(KERNEL_TAG)
endif

ifeq ($(CONTAINERIZED_BUILD),0)
OUTDIR = target/release
else
OUTDIR = out
endif

.PHONY: all
all: gen build fmt lint

.PHONY: gen
gen:
ifeq ($(CONTAINERIZED_BUILD),0)
	# Try to generate vmlinux.h. If not possible, download the newest one
	# from the libbpf community.
	$(BPFTOOL) btf dump file \
		/sys/kernel/btf/vmlinux format c > \
		src/bpf/vmlinux.h || \
		$(CURL) -L https://raw.githubusercontent.com/libbpf/libbpf-bootstrap/master/vmlinux/vmlinux_508.h \
		--output src/bpf/vmlinux.h
	$(CARGO) libbpf build --clang-path $(CLANG)
	$(CARGO) libbpf gen
else
	$(CRUNTIME) build \
		--build-arg USER_ID=$(shell id -u) \
		--build-arg GROUP_ID=$(shell id -g) \
		--target gen \
		--tag lockc-gen \
		$(EXTRA_DOCKER_FLAGS) \
		.
	$(CRUNTIME) run \
		--rm -i \
		--user "$(shell id -u):$(shell id -g)" \
		-v $(shell pwd):/usr/local/src/lockc \
		lockc-gen
endif

.PHONY: build
build:
ifeq ($(CONTAINERIZED_BUILD),0)
	$(CARGO) build --release
else
	$(CRUNTIME) build \
		--target artifact \
		--output type=local,dest=out \
		$(EXTRA_DOCKER_FLAGS) \
		.
endif

.PHONY: fmt
fmt:
ifeq ($(CONTAINERIZED_BUILD),0)
	$(CARGO) fmt
else
	$(CRUNTIME) build \
		--build-arg USER_ID=$(shell id -u) \
		--build-arg GROUP_ID=$(shell id -g) \
		--target rustfmt \
		--tag lockc-rustfmt \
		$(EXTRA_DOCKER_FLAGS) \
		.
	$(CRUNTIME) run \
		--rm -i \
		--user "$(shell id -u):$(shell id -g)" \
		-v $(shell pwd):/usr/local/src/lockc \
		lockc-rustfmt
endif

.PHONY: lint
lint:
ifeq ($(CONTAINERIZED_BUILD),0)
	$(CARGO) clippy -- -D warnings
else
	$(CRUNTIME) build \
		--build-arg USER_ID=$(shell id -u) \
		--build-arg GROUP_ID=$(shell id -g) \
		--target clippy \
		--tag lockc-clippy \
		$(EXTRA_DOCKER_FLAGS) \
		.
	$(CRUNTIME) run \
		--rm -i \
		--user "$(shell id -u):$(shell id -g)" \
		-v $(shell pwd):/usr/local/src/lockc \
		lockc-clippy
endif

.PHONY: install
install:
ifeq ($(CONTAINERIZED_BUILD),0)
	# Do not install the unit file in the OCI artifact. Keep only binaries
	# there.
	install -D -m 644 contrib/systemd/lockcd.service $(UNITDIR)/lockcd.service
endif
	install -D -m 755 $(OUTDIR)/lockcd $(BINDIR)/lockcd
	install -D -m 755 $(OUTDIR)/lockc-runc-wrapper $(BINDIR)/lockc-runc-wrapper
