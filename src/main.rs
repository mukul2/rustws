use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};

use warp::{Filter, ws::{Message, WebSocket}};

#[derive(Clone)]
struct PubSub {
    sender: Arc<Mutex<Sender<String>>>,
}

impl PubSub {
    fn new() -> Self {
        let (sender, _): (Sender<String>, Receiver<String>) = mpsc::channel();
        PubSub { sender: Arc::new(Mutex::new(sender)) }
    }

    fn subscribe(&self) -> Receiver<String> {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let receiver = sender.lock().unwrap();
            while let Ok(msg) = receiver.recv() {
                let _ = tx.send(msg);
            }
        });
        rx
    }
}

async fn handle_connection(ws: WebSocket, pubsub: PubSub) {
    let (mut tx, _) = ws.split();
    let mut receiver = pubsub.subscribe();

    while let Ok(msg) = receiver.recv() {
        let _ = tx.send(Message::text(msg)).await;
    }
}

#[tokio::main]
async fn main() {
    let pubsub = PubSub::new();

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let pubsub = pubsub.clone();
            ws.on_upgrade(move |socket| handle_connection(socket, pubsub))
        });

    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 3030)).await;
}