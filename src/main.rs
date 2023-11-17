use rust_websocket_server::{*, websocket::ConnTrait};

#[tokio::main]
async fn main() {   
    let mut websocket_conn: Box<dyn ConnTrait> = websocket::Config::new();
    websocket_conn.init("host_ip", "host_port").await;
}