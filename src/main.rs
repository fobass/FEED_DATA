use FEED_DATA::listener::listen_for_price_changes;
use FEED_DATA::models::instrument::UpdatePayload;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
use FEED_DATA::{repository, api::api};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use serde::Serialize;
use actix_cors::Cors;
use tokio::sync::broadcast;
use lazy_static::lazy_static;
use local_ip_address::local_ip;

lazy_static! {
    static ref TX: broadcast::Sender<String> = broadcast::channel(10).0;
}

#[derive(Serialize)]
pub struct Response {
    pub message: String,
}
#[get("/health")]
async fn healthcheck() -> impl Responder {
    let response = Response {
        message: "Everything is working fine".to_string(),
    };
    HttpResponse::Ok().json(response)
}

async fn not_found() -> Result<HttpResponse> {
    let response = Response {
        message: "Resource not found".to_string(),
    };
    Ok(HttpResponse::NotFound().json(response))
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr: String;
    let port = "8080";
    let feed_data = repository::database::Database::new();
    
    match local_ip() {
        Ok(ip) => {
            let ip_str = ip.to_string(); 
            addr = ip_str.clone() +":"+ port;    
        },
        Err(_e) => {
            addr = "192.168.0.101:8080".to_string();  
        },
    }

    let (bcast_tx, _bcast_rx) = channel(16);
    let app_data = web::Data::new(feed_data);
    let server = HttpServer::new(move || {
        App::new()
        .app_data(app_data.clone())
        .configure(api::config)
        .service(healthcheck)
        .default_service(web::route().to(not_found))
        .wrap(actix_web::middleware::Logger::default())
        .wrap(Cors::permissive())
    })
    .bind(addr.clone())?;

    println!("Feed server running at http://{}", addr.clone());
    tokio::spawn(server.run());

    tokio::spawn(start_websocket_server(bcast_tx.clone()));
    
    tokio::spawn(listen_for_price_changes(bcast_tx.clone()));

    tokio::signal::ctrl_c().await.expect("Failed to wait for Ctrl+C");
    println!("Shutting down...");

    Ok(())
}

async fn start_websocket_server(bcast_tx: Sender<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr: String;
    match local_ip() {
        Ok(ip) => {
            let ip_str = ip.to_string(); 
            addr = ip_str.clone() +":1092";        
        },
        Err(_e) => {
            addr = "192.168.0.101:1092".to_string();  
        },
    }
    let listener = TcpListener::bind(addr.clone()).await?;
    println!("Price update listening on {:?}", addr.clone());
    
    loop {
        let (socket, _) = listener.accept().await?;
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(ws_stream, bcast_tx).await
        });
    }
}


async fn handle_connection(mut ws_stream: WebSocketStream<TcpStream>, bcast_tx: Sender<String>) -> Result<(), Box<dyn Error + Send + Sync>> {

    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(data) = msg.as_text() {
                            let payload: UpdatePayload = serde_json::from_str(data).unwrap();
                            // if payload.client_id == "OMS_SERVER" {
                                println!("from not {:?}", payload);
                                let playload_json = serde_json::to_string(&payload.instrument)?;
                                bcast_tx.send(playload_json.to_string())?;
                            // }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            msg = bcast_rx.recv() => {
                println!("from not {:?}", msg);
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}


