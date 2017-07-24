use chrono::prelude::{DateTime, Utc};
use codec::{ToJSON, ToMessagePack};
use sender::*;

pub type UtcDateTime = DateTime<Utc>;

pub struct FluentLogger<S: Sender> {
    sender: S,
}

impl<S: Sender> FluentLogger<S> {

    pub fn log_json<T: ToJSON>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.log_json_with_timestamp(tag, Utc::now(), data)
    }

    pub fn log_json_with_timestamp<T: ToJSON>(&mut self, tag: &str, timestamp: UtcDateTime, data: &T) -> Result<(), SenderError> {
        let json = data.encode();
        let message = format!(r#"["{}",{},{}]"#, tag, timestamp.timestamp(), json);

        self.sender.emit(message.as_bytes())
    }

    pub fn log_msgpack<T: ToMessagePack>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.log_msgpack_with_timestamp(tag, Utc::now(), data)
    }

    pub fn log_msgpack_with_timestamp<T: ToMessagePack>(&mut self, tag: &str, timestamp: UtcDateTime, data: &T) -> Result<(), SenderError> {
        let mut buf: Vec<u8> = Vec::new();

        // start array
        buf.push(0x93);

        // write tag
        msgpack_util::write_string(tag, &mut buf);
        // write timestamp
        msgpack_util::write_i64(timestamp.timestamp(), &mut buf);

        // write data
        let mut pack = data.encode();
        buf.append(&mut pack);

        self.sender.emit(buf.as_slice())
    }
}

/// Send messages to fluentd via JSON encoding.
pub struct JSONLogger<S: Sender> {
    logger: FluentLogger<S>,
}

impl<S: Sender> JSONLogger<S> {

    pub fn new(underlying: FluentLogger<S>) -> JSONLogger<S> {
        JSONLogger { logger: underlying }
    }

    pub fn log<T: ToJSON>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.logger.log_json(tag, data)
    }

    pub fn log_with_timestamp<T: ToJSON>(&mut self, tag: &str, timestamp: UtcDateTime, data: &T) -> Result<(), SenderError> {
        self.logger.log_json_with_timestamp(tag, timestamp, data)
    }
}

/// Send messages to fluentd via MessagePack encoding.
pub struct MessagePackLogger<S: Sender> {
    logger: FluentLogger<S>,
}

impl<S: Sender> MessagePackLogger<S> {

    pub fn new(underlying: FluentLogger<S>) -> MessagePackLogger<S> {
        MessagePackLogger { logger: underlying }
    }

    pub fn log<T: ToMessagePack>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.logger.log_msgpack(tag, data)
    }

    pub fn log_with_timestamp<T: ToMessagePack>(&mut self, tag: &str, timestamp: UtcDateTime, data: &T) -> Result<(), SenderError> {
        self.logger.log_msgpack_with_timestamp(tag, timestamp, data)
    }
}

pub mod factory {
    //! This module provides convenient functions to instantiate fluent loggers for default use cases.
    //!
    //! # Examples
    //!
    //! ```
    //! use fluent::logger::factory;
    //!
    //! let _ = factory::json("127.0.0.1:24224");
    //! let _ = factory::msgpack("127.0.0.1:24224");
    //! ```
    use ::logger::{JSONLogger, MessagePackLogger, FluentLogger};
    use ::sender::{ConstantDelay, ErrorHandler, NullHandler, TcpSender};
    use std::io::{Error as IOError};

    pub fn json(addr: &str) -> Result<JSONLogger<TcpSender<&str, ConstantDelay, NullHandler>>, IOError> {
        TcpSender::new(addr, ConstantDelay::new(), NullHandler).map(|sender| {
            JSONLogger::new(FluentLogger { sender: sender })
        })
    }

    pub fn json_with_error_handler<H: ErrorHandler>(addr: &str, handler: H) -> Result<JSONLogger<TcpSender<&str, ConstantDelay, H>>, IOError> {
        TcpSender::new(addr, ConstantDelay::new(), handler).map(|sender| {
            JSONLogger::new(FluentLogger { sender: sender })
        })
    }

    pub fn msgpack(addr: &str) -> Result<MessagePackLogger<TcpSender<&str, ConstantDelay, NullHandler>>, IOError> {
        TcpSender::new(addr, ConstantDelay::new(), NullHandler).map(|sender| {
            MessagePackLogger::new(FluentLogger { sender: sender })
        })
    }

    pub fn msgpack_with_error_handler<H: ErrorHandler>(addr: &str, handler: H) -> Result<MessagePackLogger<TcpSender<&str, ConstantDelay, H>>, IOError> {
        TcpSender::new(addr, ConstantDelay::new(), handler).map(|sender| {
            MessagePackLogger::new(FluentLogger { sender: sender })
        })
    }
}

mod msgpack_util {
    //! A private module that provides functions to encode data to MessagePack.

    pub fn write_i64(i: i64, out: &mut Vec<u8>) {
        out.push(0xd3);
        out.push((i >> 56) as u8);
        out.push((i >> 48) as u8);
        out.push((i >> 40) as u8);
        out.push((i >> 32) as u8);
        out.push((i >> 24) as u8);
        out.push((i >> 16) as u8);
        out.push((i >> 8) as u8);
        out.push(i as u8);
    }

    pub fn write_string(s: &str, out: &mut Vec<u8>) {
        let len = s.len();

        // write str length
        if len < 32 {
            out.push((0xa0 | len) as u8);
        } else if len < 256 {
            out.push(0xd9 as u8);
            out.push(len as u8);
        } else if len < 65536 {
            out.push(0xda as u8);
            out.push((len >> 8) as u8);
            out.push(len as u8);
        } else {
            out.push(0xdb as u8);
            out.push((len >> 24) as u8);
            out.push((len >> 16) as u8);
            out.push((len >> 8) as u8);
            out.push(len as u8);
        }

        // write data
        out.extend_from_slice(s.as_bytes());
    }
}
