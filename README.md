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

```rs
struct ExampleFilter;

/// A new HttpContext gets created for each HTTP request. If we need to pass custom configuartion
/// to the filter, then we need to implement RootContext also. RootContext gets created per each
/// worker thread per plugin.
impl HttpContext for ExampleFilter {
    // This callback is invoked when the filter intercepts HTTP headers of a request from downstream.
    fn on_http_request_headers(&mut self, _: usize) -> Action {
        let mut req = Request::new();
        req.set_path_type("REQUEST_PATH".to_string());
        req.set_id("123456789".to_string());
        let message = req.write_to_bytes().unwrap();

        match self.dispatch_grpc_call(
            "grpc_service",
            "api.ExampleService",
            "GenerateHeader",
            Vec::<(&str, &[u8])>::new(),
            Some(message.as_slice()),
            Duration::from_secs(5),
        ) {
            Ok(_) => info!("Successfully dispatched gRPC unary call"),
            Err(e) => info!("Error dispatching gRPC unary call {:?}", e),
        }

        Action::Pause
    }

    // This callback is invoked when the filter intercepts HTTP headers of a response from upstream.
    fn on_http_response_headers(&mut self, _: usize) -> Action {
        let mut req = Request::new();
        req.set_path_type("RESPONSE_PATH".to_string());
        req.set_id("123456789".to_string());
        let message = req.write_to_bytes().unwrap();

        match self.dispatch_grpc_call(
            "grpc_service",
            "api.ExampleService",
            "GenerateHeader",
            Vec::<(&str, &[u8])>::new(),
            Some(message.as_slice()),
            Duration::from_secs(5),
        ) {
            Ok(_) => info!("Successfully dispatched gRPC unary call"),
            Err(e) => info!("Error dispatching gRPC unary call {:?}", e),
        }

        Action::Pause
    }
}

impl Context for ExampleFilter {
    // This callback is invoked when the filter receives a response for a gRPC unary call made during
    // on_http_request_headers or on_http_response_headers. Use the status_code to handle errors.
    fn on_grpc_call_response(&mut self, token_id: u32, status_code: u32, response_size: usize) {
        info!("gRPC response received");
        info!("{}", token_id.to_string());
        info!("{}", status_code.to_string());
        info!("{}", response_size.to_string());
        match self.get_grpc_call_response_body(0, response_size) {
            Some(bytes) => {
                let response: Response = Message::parse_from_bytes(&bytes).unwrap();
                info!("response: {:?}", response);
                if response.get_path_type() == "REQUEST_PATH" {
                    self.set_http_request_header("x-request-header", Some(response.get_header()));
                    self.resume_http_request()
                } else {
                    self.set_http_response_header("x-response-header", Some(response.get_header()));
                    self.resume_http_response()
                }
            }
            None => {
                panic!("Empty response received !!!");
            }
        }
    }
}
```

### Checking the functionality with headers

1. Run the setup with `make run` after building the filter and service with `make build`.
2. Send a HTTP request. `curl localhost:9095/ -v`
3. Read the output from the request and check for x-request-header and x-response-header. Refer to the response body to check `x-request-header`(backend sends the request information as the response). Refer to the verbose of the curl for `x-response-header`.

```sh
*   Trying 127.0.0.1:9095...
* TCP_NODELAY set
* Connected to localhost (127.0.0.1) port 9095 (#0)
> GET / HTTP/1.1
> Host: localhost:9095
> User-Agent: curl/7.68.0
> Accept: */*
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< x-powered-by: Express
< content-type: application/json; charset=utf-8
< content-length: 566
< etag: W/"236-vcNCGYLtKdXkSR+TGbr/T7vT2GA"
< date: Thu, 02 Sep 2021 13:53:15 GMT
< x-envoy-upstream-service-time: 2
< x-response-header: RES059c0520-369b-4b67-8321-7a1dd004e322
< server: envoy
< 
{
  "path": "/",
  "headers": {
    "host": "localhost:9095",
    "user-agent": "curl/7.68.0",
    "accept": "*/*",
    "x-forwarded-proto": "http",
    "x-request-id": "bfae8b99-1a5e-44d9-9dc3-99b8576ff832",
    "x-request-header": "REQa8ed2ab2-8075-49e2-998e-124a7befa274",
    "x-envoy-expected-rq-timeout-ms": "15000"
  },
  "method": "GET",
  "body": "",
  "fresh": false,
  "hostname": "localhost",
  "ip": "::ffff:172.24.0.4",
  "ips": [],
  "protocol": "http",
  "query": {},
  "subdomains": [],
  "xhr": false,
  "os": {
    "hostname": "3731c4a7ea2e"
  }
* Connection #0 to host localhost left intact
}% 
```
