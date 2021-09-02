.PHONY: service filter build proto-rust proto-go clean run

# Generate stubs from proto files.
proto-rust:
	@echo "> Generating stubs for rust"
	cd filter && \
	cargo run proto

proto-go:
	@echo "> Generating stubs for golang"
	protoc --go_out=. --go_opt=paths=source_relative --go-grpc_out=. --go-grpc_opt=paths=source_relative service/proto/api.proto

# Compiling the filter as a WASM module.
filter:
	@echo "> Compiling the filter as a WASM module...."
	mkdir -p build
	cd filter && \
	cargo build --lib --target=wasm32-unknown-unknown --release && \
	cp target/wasm32-unknown-unknown/release/envoy_wasm_rust_grpc.wasm ../build/envoy_wasm_rust_grpc.wasm
	@echo "> WASM module successfully compiled"

# Compiling the gRPC service as a executable.
service: export GOOS?=linux
service: export GOARCH?=amd64
service:
	@echo "> Compiling the gRPC server....."
	mkdir -p build
	cd service && \
	GOOS=$(GOOS) GOARCH=$(GOARCH) go build -v -o ../build/
	@echo "> gRPC server successfully compiled"

# Compile filter and gRPC service at once.
build:
	@echo "> Compiling filter and service....."
	make filter
	make service

# Deleting the build artifacts.
clean:
	@echo "> Deleting build artifacts"
	rm build/service
	rm build/envoy_wasm_rust_grpc.wasm

# Running the setup using docker-compose.
run:
	@echo >&2 "************************************************************************"
	@echo >&2 " Before running the setup, build the filter and service using make build"
	@echo >&2 "************************************************************************"
	docker-compose up
