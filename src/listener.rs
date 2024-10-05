use std::env;
use futures::{stream, StreamExt};
use futures::{FutureExt, TryStreamExt};
use tokio_postgres::NoTls;
use tokio::sync::broadcast::Sender;
use dotenv::dotenv;
pub async fn listen_for_price_changes(bcast_tx: Sender<String>) {
    dotenv().ok();
    let connection_string = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let (client, mut connection) = tokio_postgres::connect(
        &connection_string,
        NoTls,
    )
    .await
    .unwrap();

    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    let stream = stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e));
    let connection = stream.forward(tx).map(|r| r.unwrap());
    tokio::spawn(connection);

    tokio::spawn(async move {
        while let Some(message) = rx.next().await {
            match message {
                tokio_postgres::AsyncMessage::Notification(n) => {
                    if let Err(e) = bcast_tx.send(n.payload().to_string()) {
                        eprintln!("Failed to send notification to clients: {:?}", e);
                    }
                },
                _ => {}
            }
        }
    });

    if let Err(e) = client.execute("LISTEN last_price_change", &[]).await {
        eprintln!("Error starting to listen: {}", e);
        return;
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}