#![feature(test)]

extern crate fluent;
extern crate test;

use fluent::byteutil;
use fluent::codec;
use test::test::Bencher;

//#[bench]
//fn bench_big_endian(bench: &mut Bencher) {
//    bench.iter(|| {
//        let mut buf = [0u8; 8];
//        byteutil::i64_big_endian(1234567890987654321i64, &mut buf);
//    });
//}
//
//#[bench]
//fn bench_little_endian(bench: &mut Bencher) {
//    bench.iter(|| {
//        let mut buf = [0u8; 8];
//        byteutil::i64_little_endian(1234567890987654321i64, &mut buf);
//    });
//}

extern crate serde;
extern crate serde_json;
extern crate rmp_serde;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)]
struct Person<'a> {
    name: &'a str,
    age: u8,
    phone_number: &'a str,
}

//impl<'a> Copy for Person<'a> { }

impl<'a> codec::ToJSON for Person<'a> {
    fn encode(&self) -> std::string::String {
        serde_json::to_string(self).unwrap()
    }
}

use rmp_serde::{Serializer, to_vec};

impl<'a> codec::ToMessagePack for Person<'a> {
    fn encode(&self) -> Vec<u8> {
        to_vec(self).unwrap()
    }
}

#[bench]
fn bench_json_logger(bench: &mut Bencher) {
    let j1 = Person {
        name: "John Coltrane",
        age: 42,
        phone_number: "000-0000",
    };
    let j2 = Person {
        name: "John Coltrane",
        age: 42,
        phone_number: "000-0000",
    };
    let j3 = Person {
        name: "John Coltrane",
        age: 42,
        phone_number: "000-0000",
    };

    use fluent::logger::{DefaultTcpSender, DefaultUnixSocketSender};
//    let mut raw = fluent::logger::RawFluentLogger::<DefaultTcpSender<&str>>::default_tcp_logger("127.0.0.1:24224").unwrap();
    let mut raw = fluent::logger::RawFluentLogger::<DefaultUnixSocketSender<&str>>::default_uds_logger("/Users/hokada/develop/opt-tech/v7-apps/docker/fluentd/socket/socket.sock").unwrap();
//    let mut log = fluent::logger::JSONLogger::new(raw);
    let mut log = fluent::logger::MessagePackLogger::new(raw);
    log.log("foo.bar1", &j1);

    bench.iter(|| {
//        let mut raw = fluent::logger::RawFluentLogger::<DefaultTcpSender<&str>>::default_tcp_logger("127.0.0.1:24224").unwrap();
//        let mut log = fluent::logger::JSONLogger::new(raw);

//        log.log("foo.bar1", &j1);
//        log.log("foo.bar2", &j2);
//        log.log("foo.bar3", &j3);
    });
}

//fn bench_msgpack_logger(bench: &mut Bencher) {
//
//}
