#![allow(dead_code)]
#![allow(dead_code)]
#![allow(dead_code)]
use std::io::{self, Write};

/// Streaming output handler for token-by-token display
pub struct StreamHandler {
    buffer: String,
    on_token: Box<dyn FnMut(&str)>,
}

impl StreamHandler {
    pub fn new() -> Self {
        let on_token = Box::new(|token: &str| {
            print!("{}", token);
            io::stdout().flush().unwrap();
        });

        Self {
            buffer: String::new(),
            on_token,
        }
    }

    /// Feed a new token to the stream
    pub fn feed(&mut self, token: &str) {
        self.buffer.push_str(token);
        (self.on_token)(token);
    }

    /// Get the complete response so far
    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    /// Reset the stream
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

/// Simulate streaming inference output
pub async fn stream_response(response: &str) {
    let mut handler = StreamHandler::new();

    for token in response.split_inclusive(|c: char| c.is_ascii_punctuation() || c == ' ') {
        handler.feed(token);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    println!();
}
