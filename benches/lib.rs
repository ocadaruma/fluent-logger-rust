#![feature(test)]

extern crate fluent;
extern crate test;

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

//    let mut raw = fluent::logger::RawFluentLogger::<DefaultTcpSender<&str>>::default_tcp_logger("127.0.0.1:24224").unwrap();
//    let mut log = fluent::logger::JSONLogger::new(raw);

    let mut log = fluent::logger::factory::msgpack("127.0.0.1:24224").unwrap();
    //    log.log("foo.bar1", &j1);

    bench.iter(|| {
//        let mut raw = fluent::logger::RawFluentLogger::<DefaultTcpSender<&str>>::default_tcp_logger("127.0.0.1:24224").unwrap();
//        let mut log = fluent::logger::JSONLogger::new(raw);

        let mut log = fluent::logger::factory::msgpack("127.0.0.1:24224").unwrap();
        let _ = log.log("foo.bar1", &j1);
//        let _ = log.log("foo.bar2", &j2);
//        let _ = log.log("foo.bar3", &j3);
    });
}

//fn bench_msgpack_logger(bench: &mut Bencher) {
//
//}
