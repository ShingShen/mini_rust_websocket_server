pub mod data_transfer;
pub mod handler;
pub mod webrtc;

use async_trait::async_trait;

use futures_util::stream::{SplitSink, SplitStream};

use hyper::{
    Body, 
    Request, 
    Server, 
    server::conn::AddrStream, 
    service::{make_service_fn, service_fn},
    upgrade::Upgraded
};
use std::{
    collections::HashMap, 
    convert::Infallible,
    net::SocketAddr, 
    sync::Arc
};
use tokio::sync::Mutex;
use tokio_tungstenite::{
    tungstenite::protocol::Message, 
    WebSocketStream
};

use crate::websocket::handler::{Handler, RouterTrait, Router};
use crate::websocket::data_transfer::Room;

type StreamWrite = SplitSink<WebSocketStream<Upgraded>, Message>;
type StreamRead = SplitStream<WebSocketStream<Upgraded>>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, StreamWrite>>>;

type ChatRoom = Arc<Mutex<Room>>;
type ChatRooms = Arc<Mutex<HashMap<SocketAddr, ChatRoom>>>;

pub struct Config;
impl Config {
    pub fn new() -> Box<dyn ConnTrait> {
        Box::new(
            Conn {
                ws_peers: Arc::new(Mutex::new(HashMap::new())),
                ws_rooms: Arc::new(Mutex::new(HashMap::new())),
            }
        )
    }
}

#[async_trait]
pub trait ConnTrait {
    async fn init(&mut self, ip: &str, port: &str);
}

pub struct Conn {
    ws_peers: PeerMap,
    ws_rooms: ChatRooms,
}
#[async_trait]
impl ConnTrait for Conn {
    async fn init(&mut self, ip: &str, port: &str) {                        
        let make_svc = make_service_fn(|socket: &AddrStream| {
            let peers: PeerMap = self.ws_peers.clone();
            let rooms: ChatRooms = self.ws_rooms.clone();
            let addr: SocketAddr = socket.remote_addr();
            
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| { 
                    let handler: Router = Handler::new();                    
                    handler.router(req, rooms.clone(), peers.clone(), addr)
                }))
            }
        });
    
        let addr_url: SocketAddr = format!("{}:{}", ip, port).parse().expect("address url parsed error!");
        let server = Server::bind(&addr_url).serve(make_svc);
        println!("Running Websocket Server [{}]...", addr_url);
        
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}