use std::collections::VecDeque;
use std::io::{Error as IOError, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Provides retry policy based on timestamp that error happened.
pub trait RetryPolicy {

    fn clear_errors(&mut self);

    fn record_error(&mut self, now: Instant);

    fn should_retry(&self, now: Instant) -> bool;
}

/// Provides constant-delay retry policy.
///
/// # Examples
///
/// ```
/// use fluent::sender::{ConstantDelay, RetryPolicy};
/// use std::time::{Duration, Instant};
///
/// let mut policy = ConstantDelay::new();
/// let now = Instant::now();
///
/// // when no error
/// assert!(policy.should_retry(now));
///
/// // elapsed enough since last error
/// let last = now - Duration::from_millis(100);
/// policy.record_error(last);
/// assert!(policy.should_retry(now));
///
/// // should wait
/// assert!(!policy.should_retry(last + Duration::from_millis(10)));
/// ```
pub struct ConstantDelay {
    error_records: VecDeque<Instant>,
    max_errors: usize,
    wait: Duration,
}

impl ConstantDelay {
    pub fn new() -> ConstantDelay {
        ConstantDelay {
            error_records: VecDeque::new(),
            max_errors: 100,
            wait: Duration::from_millis(50),
        }
    }
}

impl RetryPolicy for ConstantDelay {

    fn clear_errors(&mut self) {
        self.error_records.clear();
    }

    fn record_error(&mut self, now: Instant) {
        self.error_records.push_front(now);

        if self.error_records.len() > self.max_errors {
            self.error_records.pop_back();
        }
    }

    fn should_retry(&self, now: Instant) -> bool {
        match self.error_records.back() {
            Some(last) => (now - *last) >= self.wait,
            None => true,
        }
    }
}

/// Provides feature to handle error (for example, log to local file / raise alert, etc)
pub trait ErrorHandler {

    fn handle_error(&mut self, timestamp: Instant, error: IOError, unsent_data: &[u8]);
}

/// Do nothing when error occurred.
pub struct NullHandler;

impl ErrorHandler for NullHandler {

    fn handle_error(&mut self, _: Instant, _: IOError, _: &[u8]) { /* do nothing */ }
}

/// Provides feature to send bytes to fluent-logger daemon.
pub trait Sender {

    fn emit(&mut self, data: &[u8]) -> Result<(), IOError>;
}

/// A Sender implementation via TCP.
///
/// # Examples
///
/// ```
/// use fluent::sender::{ConstantDelay, Sender, TcpSender, NullHandler};
///
/// let mut sender = TcpSender::new("127.0.0.1:24224", ConstantDelay::new(), NullHandler).unwrap();
///
/// sender.emit("[\"foo.bar\",1500564758,{\"key\":\"value\"}]".as_bytes());
/// ```
pub struct TcpSender<A: ToSocketAddrs + Copy, R: RetryPolicy, H: ErrorHandler> {
    addr: A,
    stream: TcpStream,
    retry_policy: R,
    error_handler: H,
    buffer: Vec<u8>,
}

impl<A: ToSocketAddrs + Copy, R: RetryPolicy, H: ErrorHandler> TcpSender<A, R, H> {
    pub fn new(addr: A, retry_policy: R, error_handler: H) -> Result<TcpSender<A, R, H>, IOError> {
        TcpStream::connect(addr).map(|stream| {
            TcpSender {
                addr: addr,
                stream: stream,
                retry_policy: retry_policy,
                buffer: Vec::with_capacity(8 * 1024 * 1024), // 8MB
                error_handler: error_handler,
            }
        })
    }

    fn send_with_reconnect_once(&mut self, data: &[u8]) -> Result<(), IOError> {
        match self.stream.write(data) {
            Err(_) => {
                TcpStream::connect(self.addr).map(|new_stream| {
                    self.stream = new_stream;
                })
            },
            Ok(_) => Ok(()),
        }
    }
}

impl<A: ToSocketAddrs + Copy, R: RetryPolicy, H: ErrorHandler> Sender for TcpSender<A, R, H> {
    fn emit(&mut self, data: &[u8]) -> Result<(), IOError> {
        self.stream.write(data).map(|_| ())
//        unimplemented!()
    }

//    fn flush_buffer(&mut self) -> Result<(), IOError> {
//
//    }
}
