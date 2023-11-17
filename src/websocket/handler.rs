use async_trait::async_trait;

use futures_util::StreamExt;

use hyper::{
    Body, 
    Method, 
    Request, 
    Response, 
    StatusCode, 
    upgrade::{on, Upgraded}
};
use std::{
    convert::Infallible,
    net::SocketAddr, 
    sync::Arc
};
use tokio::spawn;
use tokio_tungstenite::{
    tungstenite::{handshake::derive_accept_key, protocol::Role}, 
    WebSocketStream
};

use crate::websocket::webrtc::WebRTCStreamTransfer;

use super::ChatRooms;
use super::PeerMap;
use super::StreamWrite;
use super::StreamRead;

pub struct Handler;
impl Handler {
    pub fn new() -> Router {
        Router
    }
}

#[async_trait]
pub trait RouterTrait {
    async fn router(mut self, mut req: Request<Body>, rooms: ChatRooms, peers: PeerMap, addr: SocketAddr) -> Result<Response<Body>, Infallible>;
}

pub struct Router;
#[async_trait]
impl RouterTrait for Router {
    async fn router(mut self, mut req: Request<Body>, rooms: ChatRooms, peers: PeerMap, addr: SocketAddr) -> Result<Response<Body>, Infallible> {            
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/ws") => {
                let res_config: Response<Body> = ws_setting(&req); 
                spawn(async move {
                    match on(&mut req).await {
                        Ok(upgraded) => {                        
                            println!("New Websocket connection: {}", addr);                                        
                            let ws_stream: WebSocketStream<Upgraded> = WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await;    
                            let (write, read): (StreamWrite, StreamRead) = ws_stream.split();

                            peers.lock().await.insert(addr, write);
                                
                            spawn(WebRTCStreamTransfer::response_msg(Arc::clone(&peers), Arc::clone(&rooms), read, addr));
                        }
                        Err(e) => eprintln!("handle upgrade error: {}", e),
                    }
                });
                Ok(res_config)
            },
            
            _ => {
                let mut not_found: Response<Body> = Response::default();
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }
}

fn ws_setting(req: &Request<Body>) -> Response<Body> {
    let res: Response<Body> = Response::builder()
    .status(StatusCode::SWITCHING_PROTOCOLS)
    .header("Upgrade", "websocket")
    .header("Connection", "Upgrade")
    .header("Sec-WebSocket-Accept", derive_accept_key(req.headers().get("Sec-WebSocket-Key").expect("Sec-WebSocket-Key error!").as_bytes()))
    .body(Body::empty()).expect("response body error!");
    res
}

#[cfg(test)]
mod tests {
    use hyper::Response;
    use hyper::{Request, Body, StatusCode};
    use tokio_tungstenite::tungstenite::http::HeaderValue;

    use crate::websocket::handler::ws_setting;

    #[tokio::test]
    async fn test_response_config_ws_setting() {
        let mut req: Request<Body> = Request::new(Body::empty());
        req.headers_mut().insert("Sec-WebSocket-Key", HeaderValue::from_static("test-key"));
        
        let ws_setting: Response<Body> = ws_setting(&req);
        assert_eq!(ws_setting.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(ws_setting.headers().get("Upgrade"), Some(&HeaderValue::from_static("websocket")));
        assert_eq!(ws_setting.headers().get("Connection"), Some(&HeaderValue::from_static("Upgrade")));

        let body_bytes = hyper::body::to_bytes(ws_setting.into_body()).await.unwrap().to_vec();
        let vec: Vec<u8> = Vec::new();
        assert_eq!(body_bytes, vec);
    }
}