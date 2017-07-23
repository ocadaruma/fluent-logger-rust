use std::string::String;

pub trait ToMessagePack {
    fn encode(&self) -> Vec<u8>;
}

pub trait ToJSON {
    fn encode(&self) -> String;
}
