.DEFAULT_GOAL := dev
CARGO ?= cargo
export CARGO
ifeq ($(OS),Windows_NT)
PYTHON ?= python
else
PYTHON ?= python3
endif
.PHONY: help doctor setup dev build local run start check test lint fmt fmt-check
help doctor setup dev build local run start:
	"$(PYTHON)" scripts/project.py $@
check: fmt-check lint test
test:
	$(CARGO) test --locked
lint:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings
fmt:
	$(CARGO) fmt
fmt-check:
	$(CARGO) fmt --check
