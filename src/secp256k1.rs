use secp256k1::{constants, Error, Message};
use crate::utils::from_hex;

pub fn message_from_str(s: &str) ->  Result<Message, Error> {
    let mut res = [0u8; constants::MESSAGE_SIZE];
    match from_hex(s, &mut res) {
        Ok(x) => secp256k1::Message::from_slice(&res[0..x]),
        _ => Err(Error::InvalidMessage)
    }
}
