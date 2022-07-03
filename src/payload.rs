use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
pub enum PayloadType {
    Blockchain,
}

#[derive(Debug, Serialize, Deserialize)]
/// Payload for socket.
pub struct Payload {
    /// Type for payload.
    pub r#type: PayloadType,

    /// Data for payload.
    pub data: String,
}

impl Payload {
    /// Returns message to send
    pub fn serialize<T: Serialize>(r#type: PayloadType, data: &T) -> Message {
        let payload = Payload {
            r#type,
            data: serde_json::to_string(&data).unwrap()
        };
        Message::Text(serde_json::to_string(&payload).unwrap())
    }

    /// Returns deserialized payload from message
    pub fn deserialize(message: Message) -> Payload {
        serde_json::from_str::<Payload>(message.into_text().unwrap().as_str()).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::Block;
    use super::*;

    #[test]
    fn test_serialize() {
        let blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        )];
        let message = Payload::serialize(PayloadType::Blockchain, &blockchain);
        assert!(message.is_text());
    }

    #[test]
    fn test_deserialize() {
        let blockchain = vec![Block::new(
            0,
            "41CDDA1F3F0F6BD2497997A6BBAB3188090B0404C1DA5FC854C174DD42CEFD2D".to_string(),
            "".to_string(),
            1465154705,
            "block".to_string(),
            0,
            0,
        )];
        let message = Payload::serialize(PayloadType::Blockchain, &blockchain);
        assert_eq!(Payload::deserialize(message).data, serde_json::to_string(&blockchain).unwrap());
    }
}
