# Envoy proxy WASM filter with gRPC unary call (rust-sdk)

## About

In this example, envoy WASM filter makes gRPC unary calls to an external gRPC service from both request path and response path.
The external gRPC service sends generated headers as the response and the filter sets the headers from both request path and response path.

> Note: In the following request and response flow diagrams, having `[gRPC service]` in the flow does not mean that the request goes through the service.
It simply means that the filter makes an external gRPC unary call and waits untill the response. 

### Request path

`[Downstream] -> [Filter] -> [gRPC service] -> [Filter] -> [Upstream]`

### Response path

`[Upstream] -> [Filter] -> [gRPC service] -> [Filter] -> [Downstream]`

## Prerequisites

* For compiling the filter as a WASM module.
    1. Rust with WASM build target (`rustup target add wasm32-unknown-unknown` to add wasm)
    2. Protoc for generate stubs.

* For compiling the service.
    1. Golang
    2. Protoc, protoc-gen-go, protoc-gen-go-grpc

* For running the setup.
    1. Docker
    2. Docker-compose

## Usage

### Building the filter and service

```sh
make build
```

**Compiling the filter only**

```sh
make filter
```

**Compiling the service only**

```sh
make service
```

### Running the setup with docker-compose

After executing `make build`, run

```sh
make run
```

### Generating stubs from proto

```sh
make proto-rust
make proto-go
```

### Filter struct code snippet

