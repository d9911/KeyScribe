.DEFAULT_GOAL := dev

CARGO ?= cargo

.PHONY: help dev build run start check test lint fmt fmt-check

help:
	@echo "make / make dev  - Build and run the development version"
	@echo "make build       - Build the native release package and ZIP"
	@echo "make run         - Open the existing release without rebuilding"
	@echo "make start       - Build the release, then open it"
	@echo "make check       - Check formatting, lint and run tests"
	@echo "make test        - Run tests"
	@echo "make lint        - Run Clippy with warnings treated as errors"
	@echo "make fmt         - Format Rust sources"
	@echo "make fmt-check   - Check Rust formatting without changing files"

dev:
	$(CARGO) run --locked

# A prerequisite completes before the recipe, including with make -j.
start: build

check: fmt-check lint test

test:
	$(CARGO) test --locked

lint:
	$(CARGO) clippy --locked --all-targets --all-features -- -D warnings

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

ifeq ($(OS),Windows_NT)
SHELL := cmd.exe
.SHELLFLAGS := /C
POWERSHELL ?= powershell.exe

build:
	$(POWERSHELL) -NoProfile -File scripts/build-windows.ps1

run start:
	@if not exist "dist\windows-x64\Keyscribe.exe" (echo Build is missing. Run: make build & exit /b 1)
	start "" "dist\windows-x64\Keyscribe.exe"
else
SHELL := /bin/sh

ifeq ($(shell uname -s),Darwin)
build:
	sh scripts/build-macos.sh

run start:
	@test -x dist/Keyscribe.app/Contents/MacOS/keyscribe || { echo "Build is missing. Run: make build" >&2; exit 1; }
	open dist/Keyscribe.app
else
build run start:
	@echo "Keyscribe release builds and launches require macOS or Windows." >&2
	@exit 1
endif
endif
