#![allow(unused)]
use std::collections::HashSet;
use std::sync::Arc;

use vienna::random_name;
use anyhow::anyhow;
use futures_util::{lock::Mutex, StreamExt};
use futures_util::sink::SinkExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Sender},
};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
const HELP_MSG: &str = include_str!("help.txt");

#[derive(Clone)]

struct Names (Arc<Mutex<HashSet<String>>>);
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("localhost:8080").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    loop {
        let (tcp, _) = server.accept().await?;
        // spawn a separate task for
        // to handle every connection
        tokio::spawn(handle_user(tcp, tx.clone()));
    }
}

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
    let name = random_name();
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();
    sink.send(format!("You are {name}")).await?;
    sink.send(HELP_MSG).await?;
    loop {
        tokio::select! {
            Some(Ok(mut user_msg)) = stream.next() => {
                if user_msg.starts_with("/help") {
                    sink.send(HELP_MSG).await.unwrap();
                } else if user_msg.starts_with("/quit") {
                    break;
                } else {
                    user_msg.push_str("❤️");
                    tx.send(format!("{name}: {user_msg}")).unwrap();
                }
            },
            Ok(peer_msg) = rx.recv() => {
                sink.send(peer_msg).await;
            }
        }
    }
    Ok(())
}

/*    while let Some(Ok(mut user_msg)) = stream.next().await {
        tx.send(user_msg.clone());
        if user_msg.starts_with("/help") {
            sink.send(HELP_MSG).await?;
        } else if user_msg.starts_with("/quit") {
            break;
        } else {
            let peer_msg = rx.recv().await?;
            sink.send(peer_msg).await?;
        }
    }
    Ok(())
}

 */
