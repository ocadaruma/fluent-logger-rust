use byteutil;
use codec::{ToJSON, ToMessagePack};
use sender::*;
use std::io::{Error as IOError};
use std::net::ToSocketAddrs;
use std::time::Instant;

pub struct RawFluentLogger<S: Sender> {
    sender: S,
}

pub type DefaultTcpSender<A> = TcpSender<A, ConstantDelay, NullHandler>;
pub type DefaultUnixSocketSender<A> = UnixSocketSender<A, ConstantDelay, NullHandler>;

use std::convert::AsRef;
use std::path::Path;
impl<S: Sender> RawFluentLogger<S> {

    pub fn default_tcp_logger<A: ToSocketAddrs + Copy>(addr: A) -> Result<RawFluentLogger<DefaultTcpSender<A>>, IOError> {
        TcpSender::new(addr, ConstantDelay::new(), NullHandler).map(|sender| {
            RawFluentLogger { sender: sender }
        })
    }


    pub fn default_uds_logger<A: AsRef<Path> + Copy>(addr: A) -> Result<RawFluentLogger<DefaultUnixSocketSender<A>>, IOError> {
        DefaultUnixSocketSender::new(addr, ConstantDelay::new(), NullHandler).map(|sender| {
            RawFluentLogger { sender: sender }
        })
    }

    pub fn log_json<T: ToJSON>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.log_json_with_timestamp(tag, Instant::now(), data)
    }

    pub fn log_json_with_timestamp<T: ToJSON>(&mut self, tag: &str, timestamp: Instant, data: &T) -> Result<(), SenderError> {
        let json = data.encode();
        let time_str = "1500564758";
        let message = format!(r#"["{}",{},{}]"#, tag, time_str, json);

        self.sender.emit(message.as_bytes())
    }

    pub fn log_msgpack<T: ToMessagePack>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.log_msgpack_with_timestamp(tag, Instant::now(), data)
    }

    pub fn log_msgpack_with_timestamp<T: ToMessagePack>(&mut self, tag: &str, timestamp: Instant, data: &T) -> Result<(), SenderError> {
        let mut buf: Vec<u8> = Vec::new();

        // start array
        buf.push(0x93);

        // write tag
        buf.push((0xa0 | tag.len()) as u8);
        buf.extend_from_slice(tag.as_bytes());

        // write timestamp
        let mut time_bytes = [0u8; 8];
        byteutil::i64_big_endian(1500564758i64, &mut time_bytes);
        buf.push(0xd3);
        buf.extend_from_slice(&time_bytes);

        // write data
        let mut pack = data.encode();
        buf.append(&mut pack);

        self.sender.emit(buf.as_slice())
    }
}

pub struct JSONLogger<S: Sender> {
    logger: RawFluentLogger<S>,
}

impl<S: Sender> JSONLogger<S> {

    pub fn new(underlying: RawFluentLogger<S>) -> JSONLogger<S> {
        JSONLogger { logger: underlying }
    }

    pub fn log<T: ToJSON>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.logger.log_json(tag, data)
    }

    pub fn log_with_timestamp<T: ToJSON>(&mut self, tag: &str, timestamp: Instant, data: &T) -> Result<(), SenderError> {
        self.logger.log_json_with_timestamp(tag, timestamp, data)
    }
}

pub struct MessagePackLogger<S: Sender> {
    logger: RawFluentLogger<S>,
}

impl<S: Sender> MessagePackLogger<S> {

    pub fn new(underlying: RawFluentLogger<S>) -> MessagePackLogger<S> {
        MessagePackLogger { logger: underlying }
    }

    pub fn log<T: ToMessagePack>(&mut self, tag: &str, data: &T) -> Result<(), SenderError> {
        self.logger.log_msgpack(tag, data)
    }

    pub fn log_with_timestamp<T: ToMessagePack>(&mut self, tag: &str, timestamp: Instant, data: &T) -> Result<(), SenderError> {
        self.logger.log_msgpack_with_timestamp(tag, timestamp, data)
    }
}
