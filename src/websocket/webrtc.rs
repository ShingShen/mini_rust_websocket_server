use crate::websocket::data_transfer::{DataTransfer, find_room};
use crate::websocket::data_transfer::{DataType, StoreRoom};

use futures_util::StreamExt;
use std::net::SocketAddr;

use super::ChatRoom;
use super::ChatRooms;
use super::PeerMap;
use super::StreamRead;

pub struct WebRTCStreamTransfer;
impl WebRTCStreamTransfer {
    pub async fn response_msg(peers: PeerMap, mut rooms: ChatRooms, mut read: StreamRead, addr: SocketAddr) {
        while let Some(raw_msg) = read.next().await {
            match raw_msg {
                Ok(msg) => { 
                    println!("client message from [{}]: {}", addr, msg);    
                    let msg_string: String = msg.to_string();                    
                    let raw_data: Result<StoreRoom, _> = serde_json::from_str(&msg_string);
                    match raw_data {
                        Ok(data) => {
                            if msg.is_text() || msg.is_binary() { 
                                let room: Option<ChatRoom> = find_room(&mut rooms, data.room_id.clone()).await;
                                match data.data_type.as_str() {
                                    "store_room" => DataType::store_room(rooms.clone(), room, data, addr).await,
                                    "store_offer" => DataType::store_offer(rooms.clone(), room, data).await,
                                    "store_candidate" => DataType::store_candidate(rooms.clone(), room, data).await,
                                    "send_answer" => DataType::send_answer(room, data, peers.clone()).await,
                                    "send_candidate" => DataType::send_candidate(room, data, peers.clone()).await,
                                    "join_call" => DataType::join_call(room, peers.clone()).await,
                                    _ => eprintln!("Data type is incorrect!"),
                                }
                            }
                        },
                        Err(err_msg) => eprintln!("store room data type error: {}", err_msg)
                    }

                    if msg.is_close() {
                        DataType::close(rooms.clone(), peers.clone(), addr).await;
                    }
                }
                Err(e) => {
                    DataType::close(rooms.clone(), peers.clone(), addr).await;
                    eprintln!("an error occured while processing incoming messages: {}", e);
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_webrtc_response_msg() {}
}