pub mod messages {
    use std::{fmt, convert::{TryFrom, TryInto}};

    use tungstenite::Message;


    #[derive(Debug, Clone)]
    pub struct UnknownMessageError;
    impl fmt::Display for UnknownMessageError {
        fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "unknown message type")
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum MessageType {
        GetEntitiesRequest,
        NewEntityRequest,
    }

    const MessageIndex: [MessageType; 2] = [MessageType::GetEntitiesRequest, MessageType::NewEntityRequest];
    
    #[derive(Debug)]
    pub struct RawMessage {
        pub data: Vec<u8>
    }
    impl RawMessage {
        pub fn getMsgType(&self) -> Result<MessageType, UnknownMessageError> {
            self.data.first()
                .ok_or(UnknownMessageError)
                .and_then(|id| {
                    Ok(MessageIndex[*id as usize])
                })  
        }
    }
    impl TryFrom<Message> for RawMessage {
        type Error = &'static str;
        fn try_from(value: Message) -> Result<Self, Self::Error> {
            match value {
                Message::Binary(data) => {
                    Ok(RawMessage { data })
                },
                _ => {
                    Err("Only Binary message types can be converted")
                }
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct GetEntitiesRequestMessage {
        x: u32, // 1-4th bytes
        y: u32, // 5-8th bytes
        w: u32, // 9-12th bytes
        h: u32, // 13-16th bytes
    } // = 1 + 16 bytes = 17 bytes 
    impl TryFrom<RawMessage> for GetEntitiesRequestMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 17 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                let x = u32::from_be_bytes(rm.data[1..5].try_into().unwrap());
                let y = u32::from_be_bytes(rm.data[5..9].try_into().unwrap());
                let w = u32::from_be_bytes(rm.data[9..13].try_into().unwrap());
                let h = u32::from_be_bytes(rm.data[13..17].try_into().unwrap());
                Ok(GetEntitiesRequestMessage { x, y, w, h })
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct NewEntityRequestMessage {
        x: u32,         // 1-4th bytes
        y: u32,         // 5-8th bytes
        entity_id: u8,  // 9th byte
    } // = 1 + 9 bytes = 10 bytes 
    impl TryFrom<RawMessage> for NewEntityRequestMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 10 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                let x = u32::from_be_bytes(rm.data[1..5].try_into().unwrap());
                let y = u32::from_be_bytes(rm.data[5..9].try_into().unwrap());
                let entity_id = rm.data[9];
                Ok(NewEntityRequestMessage { x, y, entity_id })
            }
        }
    }


}