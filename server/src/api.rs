use crate::*;
use std::sync::{atomic::Ordering, Arc};
use warp::ws::{Message, WebSocket};
use warp::{filters::BoxedFilter, Filter, Reply};

use futures::stream::SplitSink;
use futures::SinkExt;
use futures::StreamExt;

async fn send_blocks(
    client: Arc<Client>,
    state: Arc<State>,
    solve_state: Arc<ScheduleState>,
    mut ws_tx: SplitSink<warp::ws::WebSocket, warp::ws::Message>,
    notify: Arc<tokio::sync::Notify>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error>> {
    loop {
        notify.notified().await;
        let client_buffer_size = state.client_buffer_size.load(Ordering::Relaxed);
        for _ in 0..(client_buffer_size - client.claimed_len()) {
            let block = solve_state.get_block(&client).await?;
            let message = bincode::serialize(&block)?;
            client.add_sent_bytes(message.len());
            ws_tx.send(Message::binary(message)).await?;
        }
    }
}

async fn client_connected(mut ws: WebSocket, state: Arc<State>) {
    let client = Arc::new(Client::new(
        state.next_client_id.fetch_add(1, Ordering::Relaxed),
    ));
    println!("New client: {}", client.get_id());

    let init = ws.next().await;
    let init = if let Some(Ok(init)) = init {
        init
    } else {
        return;
    };
    let init = init.as_bytes();
    client.add_recieved_bytes(init.len());
    let arg: schedule_util::ScheduleArg = if let Some(init) = bincode::deserialize(init).ok() {
        init
    } else {
        return;
    };
    let arg = Arc::new(arg);
    let solve_state = state.get_schedule_solve_state(arg);
    let (ws_tx, mut ws_rx) = ws.split();
    let block_sender_notify = Arc::new(tokio::sync::Notify::new());
    let block_request = send_blocks(
        client.clone(),
        state.clone(),
        solve_state.clone(),
        ws_tx,
        block_sender_notify.clone(),
    );
    block_sender_notify.notify_one();
    let solve_state_clone = solve_state.clone();
    let client_clone = client.clone();
    let input_handler = async move {
        while let Some(next) = ws_rx.next().await {
            let next = next?;
            let next = next.as_bytes();
            client_clone.add_recieved_bytes(next.len());
            if next == b"request" {
                block_sender_notify.notify_one();
            } else if let Ok(batch) = bincode::deserialize::<schedule_util::BatchOutput>(&next) {
                client_clone.add_stats(&batch.stats);
                solve_state_clone.add_batch_result(&client_clone, batch);
                block_sender_notify.notify_one();
            } else {
                println!("Unknown data: {:?}", next);
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let client_clone = client.clone();
    let timeout = async move {
        loop {
            let timeout = state.timeout.load(Ordering::Relaxed);
            let timeout = std::time::Duration::from_secs(timeout);
            tokio::time::sleep(timeout).await;
            if client_clone.get_last_updated().elapsed() > timeout {
                return Err::<(), Box<dyn std::error::Error>>(Box::new(ApiError::Timeout));
            }
        }
    };
    let result = futures::future::try_join3(block_request, input_handler, timeout).await;
    if result.is_err() {
        println!("Client {} Result: {:?}", client.get_id(), result);
    }
    println!("Disconnected client: {}", client.get_id());
    solve_state.free_all_from_client(&client);
}

pub fn get_api_filter(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    warp::ws()
        .map(move |ws: warp::ws::Ws| {
            let state = state.clone();
            ws.on_upgrade(move |websocket| client_connected(websocket, state.clone()))
        })
        .boxed()
}
