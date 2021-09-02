use log::info;
use protobuf::Message;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::Duration;

mod api;
use api::Request;
use api::Response;

#[no_mangle]
pub fn _start() {
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_http_context(|_, _| -> Box<dyn HttpContext> { Box::new(ExampleFilter) });
}

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
