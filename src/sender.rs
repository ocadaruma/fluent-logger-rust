use std::collections::VecDeque;
use std::io::{Error as IOError, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum RetryLevel {
    Ready,
    Wait,
    AttemptsExceeded,
}

/// Provides retry manager based on error timestamp.
pub trait RetryManager {

    fn reset(&mut self);

    fn record_error(&mut self, now: Instant);

    fn attempt(&self, now: Instant) -> RetryLevel;
}

/// Provides constant-delay retry manager.
///
/// # Examples
///
/// ```
/// use fluent::sender::{ConstantDelay, RetryLevel, RetryManager};
/// use std::time::{Duration, Instant};
///
/// let mut manager = ConstantDelay::new();
/// let now = Instant::now();
///
/// // when no error
/// assert_eq!(manager.attempt(now), RetryLevel::Ready);
///
/// // elapsed enough since last error
/// let last = now - Duration::from_millis(100);
/// manager.record_error(last);
/// assert_eq!(manager.attempt(now), RetryLevel::Ready);
///
/// // should wait
/// assert_eq!(manager.attempt(last + Duration::from_millis(10)), RetryLevel::Wait);
/// ```
pub struct ConstantDelay {
    error_records: VecDeque<Instant>,
    max_errors: usize,
    wait: Duration,
}

impl ConstantDelay {
    pub fn default() -> ConstantDelay {
        ConstantDelay {
            error_records: VecDeque::with_capacity(100),
            max_errors: 100,
            wait: Duration::from_millis(50),
        }
    }
}

impl RetryManager for ConstantDelay {

    fn reset(&mut self) {
        self.error_records.clear();
    }

    fn record_error(&mut self, now: Instant) {
        self.error_records.push_front(now);

        if self.error_records.len() > self.max_errors {
            self.error_records.pop_back();
        }
    }

    fn attempt(&self, now: Instant) -> RetryLevel {
        if self.error_records.len() > self.max_errors {
            RetryLevel::AttemptsExceeded
        } else {
            match self.error_records.back() {
                Some(last) if (now - *last) >= self.wait => RetryLevel::Ready,
                Some(_) => RetryLevel::Wait,
                None => RetryLevel::Ready,
            }
        }
    }
}

/// Provides feature to handle error (for example, log to local file / raise alert, etc)
pub trait ErrorHandler {

    fn handle_error(&mut self, timestamp: Instant, error: &SenderError, current_buffer: &[u8]);
}

/// Do nothing when error occurred.
pub struct NullHandler;

impl ErrorHandler for NullHandler {

    fn handle_error(&mut self, _: Instant, _: &SenderError, _: &[u8]) { /* do nothing */ }
}

/// Provides feature to send bytes to fluentd.
pub trait Sender {

    fn emit(&mut self, data: &[u8]) -> Result<(), SenderError>;

    fn flush(&mut self) -> Result<(), SenderError>;
}

pub enum SenderError {
    IO(IOError),
    TooLargeData,
    RetryAttemptsExceeded,
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
/// let _ = sender.emit("[\"foo.bar\",1500564758,{\"key\":\"value\"}]".as_bytes());
/// ```
pub struct TcpSender<A: ToSocketAddrs + Copy, R: RetryManager, H: ErrorHandler> {
    addr: A,
    stream: TcpStream,
    retry_manager: R,
    error_handler: H,
    buffer: Vec<u8>,
}

impl<A: ToSocketAddrs + Copy, R: RetryManager, H: ErrorHandler> TcpSender<A, R, H> {

    pub fn new(addr: A, retry_manager: R, error_handler: H) -> Result<TcpSender<A, R, H>, IOError> {
        TcpStream::connect(addr).map(|stream| {
            TcpSender {
                addr: addr,
                stream: stream,
                retry_manager: retry_manager,
                buffer: Vec::with_capacity(8 * 1024 * 1024), // 8MB
                error_handler: error_handler,
            }
        })
    }

    fn send_buffer_with_reconnect_once(&mut self) -> Result<(), IOError> {
        match self.stream.write(self.buffer.as_slice()) {
            Err(_) => {
                TcpStream::connect(self.addr).and_then(|new_stream| {
                    self.stream = new_stream;
                    self.stream.write(self.buffer.as_slice()).map(|_| ())
                })
            },
            Ok(_) => Ok(()),
        }
    }

    fn flush_buffer(&mut self) -> Result<(), SenderError> {
        if self.buffer.is_empty() {
            self.retry_manager.reset();
            Ok(())
        } else {
            match self.send_buffer_with_reconnect_once() {
                Err(e) => {
                    let now = Instant::now();
                    let err = SenderError::IO(e);
                    self.retry_manager.record_error(now);
                    self.error_handler.handle_error(now, &err, self.buffer.as_slice());
                    Err(err)
                },
                Ok(_) => {
                    self.buffer.clear();
                    self.retry_manager.reset();
                    Ok(())
                },
            }
        }
    }
}

impl<A: ToSocketAddrs + Copy, R: RetryManager, H: ErrorHandler> Sender for TcpSender<A, R, H> {

    fn emit(&mut self, data: &[u8]) -> Result<(), SenderError> {

        let now = Instant::now();

        if self.retry_manager.attempt(now) == RetryLevel::AttemptsExceeded {
            let error = SenderError::RetryAttemptsExceeded;
            self.error_handler.handle_error(now, &error, self.buffer.as_slice());
            Err(error) ?
        }

        // if buffer space is insufficient, flush first
        if self.buffer.len() + data.len() > self.buffer.capacity() && self.retry_manager.attempt(now) == RetryLevel::Ready {
            self.flush_buffer() ?
        }
        // if data is larger than buffer capacity, just return error.
        if data.len() > self.buffer.capacity() - self.buffer.len() {
            let error = SenderError::TooLargeData;
            self.error_handler.handle_error(now, &error, self.buffer.as_slice());
            Err(error) ?
        }

        // write to buffer then flush
        self.buffer.extend_from_slice(data);
        if self.retry_manager.attempt(now) == RetryLevel::Ready {
            self.flush_buffer()
        } else {
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<(), SenderError> {
        self.flush_buffer()
    }
}
