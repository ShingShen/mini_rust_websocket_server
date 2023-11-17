use async_trait::async_trait;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::sync::{MutexGuard, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;

use super::ChatRoom;
use super::ChatRooms;
use super::PeerMap;

pub struct DataType;

#[async_trait]
pub trait DataTransfer {
    async fn store_room(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom, addr: SocketAddr);
    async fn store_offer(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom);
    async fn store_candidate(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom);
    async fn send_answer(room: Option<ChatRoom>, data: StoreRoom, peers: PeerMap);
    async fn send_candidate(room: Option<ChatRoom>, data: StoreRoom, peers: PeerMap);
    async fn join_call(room: Option<ChatRoom>, peers: PeerMap);
    async fn close(rooms: ChatRooms, peers: PeerMap, addr: SocketAddr);
}

#[async_trait]
impl DataTransfer for DataType {
    async fn store_room(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom, addr: SocketAddr) {
        match room {
            None => {
                let new_room: Room = Room { 
                    room_id: data.room_id.clone(),
                    offer: Offer { 
                        r#type: String::new(), 
                        sdp: String::new(), 
                    },
                    candidates: Vec::new(),
                };
                rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));
                println!("= store_room = rooms: {:?}", rooms)
            },
            Some(exist_room) => {
                eprintln!("= store_room = {} exist!", exist_room.lock().await.room_id.clone())
            },
        }
    }

    async fn store_offer(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom) {
        match room {
            Some(exist_room) => {
                exist_room.lock().await.offer = data.offer;
                println!("= store_offer = rooms: {:?}", rooms);
            },
            None => eprintln!("= store_offer = The room do not exist!"),
        }
    }

    async fn store_candidate(rooms: ChatRooms, room: Option<ChatRoom>, data: StoreRoom) {
        match room {
            Some(exist_room) => {
                exist_room.lock().await.candidates.push(data.candidate);
                println!("= store_candidate = rooms: {:?}", rooms);
            },
            None => eprintln!("= store_candidate = The room do not exist!"),
        }
    }

    async fn send_answer(room: Option<ChatRoom>, data: StoreRoom, peers: PeerMap) {
        match room {
            Some(_) => {
                let answer_data: Value = json!({
                    "data_type": "answer",
                    "answer": data.answer
                });
                let answer_data_string: String = serde_json::to_string(&answer_data).expect("Failed to serialize!");
                send_to_all(peers.clone(), Message::Text(answer_data_string.clone())).await;
                
                println!("= send_answer = answer_data: {}", answer_data_string.clone());
            },
            None => eprintln!("= send_answer = The room do not exist!"),
        }
    }

    async fn send_candidate(room: Option<ChatRoom>, data: StoreRoom, peers: PeerMap) {
        match room {
            Some(_) => {
                let candidate_data: Value = json!({
                    "data_type": "candidate",
                    "candidate": data.candidate
                });
                let candidate_data_string: String = serde_json::to_string(&candidate_data).expect("Failed to serialize!");
                send_to_all(peers.clone(), Message::Text(candidate_data_string.clone())).await;

                println!("= send_candidate = candidate_data: {}", candidate_data_string.clone());
            },
            None => eprintln!("= send_candidate = The room do not exist!"),
        }
    }

    async fn join_call(room: Option<ChatRoom>, peers: PeerMap) {
        match room {
            Some(exist_room) => {
                let offer_data: Value = json!({
                    "data_type": "offer",
                    "offer": exist_room.lock().await.offer,
                });
                let offer_data_string: String = serde_json::to_string(&offer_data).expect("Failed to serialize!");
                send_to_all(peers.clone(), Message::Text(offer_data_string.clone())).await;
                println!("= join_call = offer_data: {}", offer_data_string.clone());
                
                for candidate in exist_room.lock().await.candidates.clone() {
                    let candidate_data: Value = json!({
                        "data_type": "candidate",
                        "candidate": candidate,
                    });
                    let candidate_data_string: String = serde_json::to_string(&candidate_data).expect("Failed to serialize!");
                    send_to_all(peers.clone(), Message::Text(candidate_data_string.clone())).await;
                    println!("= join_call = candidate_data: {}", candidate_data_string.clone());
                }
            },
            None => eprintln!("= join_call = The room do not exist!"),
        }
    }

    async fn close(rooms: ChatRooms, peers: PeerMap, addr: SocketAddr) {
        println!("[{}]: WebSocket connection closed!", addr);
        rooms.lock().await.remove(&addr);
        peers.lock().await.remove(&addr);

        println!("= WebSocket Closed = peers: {:?}", peers);
        println!("= WebSocket Closed = rooms: {:?}", rooms);
    }
}

pub async fn find_room(rooms: &ChatRooms, room_id: String) -> Option<ChatRoom> {
    let rooms: MutexGuard<'_, HashMap<SocketAddr, Arc<Mutex<Room>>>> = rooms.lock().await;
    for (_, room) in rooms.iter() {
        if room.lock().await.room_id == room_id {
            return Some(room.clone())
        }
    }
    None
}

async fn send_to_all(peers: PeerMap, msg: Message) {
    for (addr, write) in peers.lock().await.iter_mut() {
        println!("send to [{}]", addr);
        write.send(msg.clone()).await.expect("Failed to send msg to all!")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Offer {
    pub r#type: String,
    pub sdp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Answer {
    r#type: String,
    sdp: String,
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Candidate {
    candidate: String,
    sdpMid: String,
    sdpMLineIndex: u8,
    usernameFragment: String,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room_id: String,
    pub offer: Offer,
    pub candidates: Vec<Candidate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreRoom {
    pub data_type: String,
    pub room_id: String,
    pub offer: Offer,
    pub answer: Answer,
    pub candidate: Candidate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use hyper::{Request, Body};
    use hyper::upgrade::on;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::collections::HashMap;
    use tokio_tungstenite::WebSocketStream;

    #[tokio::test]
    async fn test_store_room() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let data = StoreRoom {
            data_type: String::from("test"),
            room_id: String::from("test_room"),
            offer: Offer {
                r#type: String::from("offer"),
                sdp: String::from("sdp_offer"),
            },
            answer: Answer {
                r#type: String::from("answer"),
                sdp: String::from("sdp_answer"),
            },
            candidate: Candidate {
                candidate: String::from("candidate"),
                sdpMid: String::from("sdpMid"),
                sdpMLineIndex: 0,
                usernameFragment: String::from("usernameFragment"),
            },
        };

        DataType::store_room(rooms.clone(), None, data.clone(), addr).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
        assert_eq!(room.unwrap().lock().await.room_id, data.clone().room_id);
    }

    #[tokio::test]
    async fn test_store_offer() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let data = StoreRoom {
            data_type: String::from("test"),
            room_id: String::from("test_room"),
            offer: Offer {
                r#type: String::from("offer"),
                sdp: String::from("sdp_offer"),
            },
            answer: Answer {
                r#type: String::from("answer"),
                sdp: String::from("sdp_answer"),
            },
            candidate: Candidate {
                candidate: String::from("candidate"),
                sdpMid: String::from("sdpMid"),
                sdpMLineIndex: 0,
                usernameFragment: String::from("usernameFragment"),
            },
        };

        let new_room: Room = Room { 
            room_id: data.room_id.clone(),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: Vec::new(),
        };
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        DataType::store_offer(rooms.clone(), rooms.lock().await.get(&addr).cloned(), data.clone()).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
        assert_eq!(room.unwrap().lock().await.offer, data.offer);
    }

    #[tokio::test]
    async fn test_store_candidate() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let data = StoreRoom {
            data_type: String::from("test"),
            room_id: String::from("test_room"),
            offer: Offer {
                r#type: String::from("offer"),
                sdp: String::from("sdp_offer"),
            },
            answer: Answer {
                r#type: String::from("answer"),
                sdp: String::from("sdp_answer"),
            },
            candidate: Candidate {
                candidate: String::from("candidate"),
                sdpMid: String::from("sdpMid"),
                sdpMLineIndex: 0,
                usernameFragment: String::from("usernameFragment"),
            },
        };

        let new_room: Room = Room { 
            room_id: data.room_id.clone(),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: Vec::new(),
        };
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        DataType::store_candidate(rooms.clone(), rooms.lock().await.get(&addr).cloned(), data.clone()).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
        assert_eq!(room.unwrap().lock().await.candidates[0], data.candidate);
    }

    #[tokio::test]
    async fn test_send_answer() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut req = Request::new(Body::empty());
        if let Ok(upgraded) = on(&mut req).await {
            let (write, _) = WebSocketStream::from_raw_socket(upgraded, tokio_tungstenite::tungstenite::protocol::Role::Client, None).await.split();
            peers.lock().await.insert(addr, write);
        }

        let new_room: Room = Room { 
            room_id: String::from("test_room"),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: Vec::new(),
        };
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        let data = StoreRoom {
            data_type: String::from("test"),
            room_id: String::from("test_room"),
            offer: Offer {
                r#type: String::from("offer"),
                sdp: String::from("sdp_offer"),
            },
            answer: Answer {
                r#type: String::from("answer"),
                sdp: String::from("sdp_answer"),
            },
            candidate: Candidate {
                candidate: String::from("candidate"),
                sdpMid: String::from("sdpMid"),
                sdpMLineIndex: 0,
                usernameFragment: String::from("usernameFragment"),
            },
        };

        DataType::send_answer(rooms.lock().await.get(&addr).cloned(), data.clone(), peers.clone()).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
    }

    #[tokio::test]
    async fn test_send_candidate() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut req = Request::new(Body::empty());
        if let Ok(upgraded) = on(&mut req).await {
            let (write, _) = WebSocketStream::from_raw_socket(upgraded, tokio_tungstenite::tungstenite::protocol::Role::Client, None).await.split();
            peers.lock().await.insert(addr, write);
        }

        let new_room: Room = Room { 
            room_id: String::from("test_room"),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: Vec::new(),
        };
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        let data = StoreRoom {
            data_type: String::from("test"),
            room_id: String::from("test_room"),
            offer: Offer {
                r#type: String::from("offer"),
                sdp: String::from("sdp_offer"),
            },
            answer: Answer {
                r#type: String::from("answer"),
                sdp: String::from("sdp_answer"),
            },
            candidate: Candidate {
                candidate: String::from("candidate"),
                sdpMid: String::from("sdpMid"),
                sdpMLineIndex: 0,
                usernameFragment: String::from("usernameFragment"),
            },
        };

        DataType::send_candidate(rooms.lock().await.get(&addr).cloned(), data.clone(), peers.clone()).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
    }

    #[tokio::test]
    async fn test_join_call() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut req = Request::new(Body::empty());
        if let Ok(upgraded) = on(&mut req).await {
            let (write, _) = WebSocketStream::from_raw_socket(upgraded, tokio_tungstenite::tungstenite::protocol::Role::Client, None).await.split();
            peers.lock().await.insert(addr, write);
        }


        let new_room: Room = Room { 
            room_id: String::from("test_room"),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: vec![
                Candidate {
                    candidate: String::from("candidate"),
                    sdpMid: String::from("sdpMid"),
                    sdpMLineIndex: 0,
                    usernameFragment: String::from("usernameFragment"),
                }
            ],
        };

        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        DataType::join_call(rooms.lock().await.get(&addr).cloned(), peers.clone()).await;

        let binding = rooms.lock().await;
        let room = binding.get(&addr);
        assert!(room.is_some());
    }

    #[tokio::test]
    async fn test_close() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut req = Request::new(Body::empty());
        if let Ok(upgraded) = on(&mut req).await {
            let (write, _) = WebSocketStream::from_raw_socket(upgraded, tokio_tungstenite::tungstenite::protocol::Role::Client, None).await.split();
            peers.lock().await.insert(addr, write);
        }

        let new_room: Room = Room { 
            room_id: String::from("test_room"),
            offer: Offer { 
                r#type: String::new(), 
                sdp: String::new(), 
            },
            candidates: Vec::new(),
        };
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(new_room.clone())));

        DataType::close(rooms.clone(), peers.clone(), addr).await;

        assert!(rooms.lock().await.get(&addr).is_none());
        assert!(peers.lock().await.get(&addr).is_none());
    }

    #[tokio::test]
    async fn test_find_room() {
        let rooms: ChatRooms = Arc::new(Mutex::new(HashMap::new()));
        let room = Room {
            room_id: "test_room".to_string(),
            offer: Offer {
                r#type: "test_offer".to_string(),
                sdp: "test_sdp".to_string(),
            },
            candidates: vec![Candidate {
                candidate: "test_candidate".to_string(),
                sdpMid: "test_sdpMid".to_string(),
                sdpMLineIndex: 0,
                usernameFragment: "test_usernameFragment".to_string(),
            }],
        };
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        rooms.lock().await.insert(addr, Arc::new(Mutex::new(room)));

        let result = find_room(&rooms, "test_room".to_string()).await;
        assert!(result.is_some());
        let binding = result.unwrap();
        let room = binding.lock().await;
        assert_eq!(room.room_id, "test_room");
        assert_eq!(room.offer.r#type, "test_offer");
        assert_eq!(room.offer.sdp, "test_sdp");
        assert_eq!(room.candidates[0].candidate, "test_candidate");
        assert_eq!(room.candidates[0].sdpMid, "test_sdpMid");
        assert_eq!(room.candidates[0].sdpMLineIndex, 0);
        assert_eq!(room.candidates[0].usernameFragment, "test_usernameFragment");
    }
}