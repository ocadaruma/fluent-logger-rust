use std::string::String;

/// Represents entity can be encoded to MessagePack.
pub trait ToMessagePack {
    fn encode(&self) -> Vec<u8>;
}

/// Represents entity can be encoded to JSON.
pub trait ToJSON {
    fn encode(&self) -> String;
}
