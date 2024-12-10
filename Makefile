CMD_DIR := cmd/

all: run

run:
	go run $(CMD_DIR)main.go

test:
	go test ./...

lint:
	golangci-lint run ./...

.PHONY: all run test lint
