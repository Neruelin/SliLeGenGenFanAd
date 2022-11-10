pub mod ws_messages {
    use std::{fmt, convert::{TryFrom, TryInto}, sync::RwLockReadGuard};
    use crate::game_state::game_state::Obj;
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
        ClearEntitiesRequest,
        MoveEntityRequest,
        CreateControlEntity,
        Disconnect,
    }

    const MESSAGE_INDEX: [MessageType; 6] = [MessageType::GetEntitiesRequest, MessageType::NewEntityRequest, MessageType::ClearEntitiesRequest, MessageType::MoveEntityRequest, MessageType::CreateControlEntity, MessageType::Disconnect];
    
    #[derive(Debug)]
    pub struct RawMessage {
        pub data: Vec<u8>
    }
    impl RawMessage {
        pub fn get_msg_type(&self) -> Result<MessageType, UnknownMessageError> {
            self.data.first()
                .ok_or(UnknownMessageError)
                .and_then(|id| {
                    Ok(MESSAGE_INDEX[*id as usize])
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
        pub x: u32, // 1-4th bytes
        pub y: u32, // 5-8th bytes
        pub w: u32, // 9-12th bytes
        pub h: u32, // 13-16th bytes
    } // = 1 + 16 bytes = 17 bytes 
    impl GetEntitiesRequestMessage {
        pub fn create_response(objs: RwLockReadGuard<Vec<Obj>>, x: u32, y: u32, w: u32, h: u32) -> Message {
            let mut resp: Vec<u8> = vec![];
            resp.push(0);
            resp.push(0);
            for o in objs.iter() {
                if o.x >= x && o.x < x + w && o.y >= y && o.y < y + h {
                    resp[1] += 1;
                    let mut temp = o.id.to_be_bytes().to_vec();
                    temp.reverse();
                    resp.append(&mut temp);
                    temp = o.x.to_be_bytes().to_vec();
                    temp.reverse();
                    resp.append(&mut temp);
                    temp = o.y.to_be_bytes().to_vec();
                    temp.reverse();
                    resp.append(&mut temp);
                }
            }
            Message::Binary(resp)
        }
    }
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
        pub x: u32,         // 1-4th bytes
        pub y: u32,         // 5-8th bytes
        pub entity_id: u32,  // 9-12th bytes
    } // = 1 + 12 bytes = 13 bytes 
    impl TryFrom<RawMessage> for NewEntityRequestMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 13 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                let x = u32::from_be_bytes(rm.data[1..5].try_into().unwrap());
                let y = u32::from_be_bytes(rm.data[5..9].try_into().unwrap());
                let entity_id = u32::from_be_bytes(rm.data[9..13].try_into().unwrap());
                Ok(NewEntityRequestMessage { x, y, entity_id })
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct ClearEntitiesRequestMessage {} // = 1 byte
    impl TryFrom<RawMessage> for ClearEntitiesRequestMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 1 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                Ok(ClearEntitiesRequestMessage {})
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct MoveEntityRequestMessage {
        pub x: i32,         // 1-4th bytes
        pub y: i32,         // 5-8th bytes
        pub entity_id: u32,  // 9-12th bytes
    } // = 1 + 12 bytes = 13 bytes 
    impl TryFrom<RawMessage> for MoveEntityRequestMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 13 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                let x = i32::from_be_bytes(rm.data[1..5].try_into().unwrap());
                let y = i32::from_be_bytes(rm.data[5..9].try_into().unwrap());
                let entity_id = u32::from_be_bytes(rm.data[9..13].try_into().unwrap());
                Ok(MoveEntityRequestMessage { x, y, entity_id })
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct CreateControlEntityMessage {} // 1 byte
    impl CreateControlEntityMessage {
        pub fn create_response(obj: Obj) -> Message {
            let mut resp: Vec<u8> = vec![];
            resp.push(4);
            let mut temp = obj.id.to_be_bytes().to_vec();
            temp.reverse();
            resp.append(&mut temp);
            temp = obj.x.to_be_bytes().to_vec();
            temp.reverse();
            resp.append(&mut temp);
            temp = obj.y.to_be_bytes().to_vec();
            temp.reverse();
            resp.append(&mut temp);
            Message::Binary(resp)
        }
    }
    impl TryFrom<RawMessage> for CreateControlEntityMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 1 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                Ok(CreateControlEntityMessage { })
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DisconnectMessage {
        pub id: u32, // 1-5th bytes
    } // 1 + 4 = 5 bytes
    impl TryFrom<RawMessage> for DisconnectMessage {
        type Error = &'static str;
        fn try_from(rm: RawMessage) -> Result<Self, Self::Error> {
            let rm_len = rm.data.len();
            if rm_len != 5 {
                println!("packetsize {:?}", rm_len);
                Err("RawMessage data is of the the wrong length")
            } else {
                let id = u32::from_be_bytes(rm.data[1..5].try_into().unwrap());
                Ok(DisconnectMessage { id })
            }
        }
    }



}