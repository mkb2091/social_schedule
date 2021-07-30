use crate::*;
use std::sync::{atomic::Ordering, Arc};
use warp::ws::{Message, WebSocket};
use warp::{filters::BoxedFilter, Filter, Reply};

use futures::SinkExt;
use futures::StreamExt;

async fn send_block(
    client: &Arc<Client>,
    solve_state: Arc<ScheduleState>,
    tx: &tokio::sync::mpsc::UnboundedSender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let block = solve_state.get_block(client).await?;
    let message = bincode::serialize(&block)?;
    tx.send(Message::binary(message))?;
    Ok(())
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
    let arg: schedule_util::ScheduleArg = if let Some(init) = bincode::deserialize(init).ok() {
        init
    } else {
        return;
    };
    let arg = Arc::new(arg);
    let solve_state = state.get_schedule_solve_state(arg);
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let forward_messages = async move {
        while let Some(next) = rx.recv().await {
            ws_tx.send(next).await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    let solve_state_clone = solve_state.clone();
    let client_clone = client.clone();
    let block_sender_notify = Arc::new(tokio::sync::Notify::new());
    let notify2 = block_sender_notify.clone();
    let state_clone = state.clone();
    let block_request = async move {
        loop {
            notify2.notified().await;
            let client_buffer_size = state_clone.client_buffer_size.load(Ordering::Relaxed);
            while client_clone.claimed_len() < client_buffer_size {
                send_block(&client_clone, solve_state_clone.clone(), &tx).await?
            }
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    block_sender_notify.notify_one();
    let solve_state_clone = solve_state.clone();
    let client_clone = client.clone();
    let input_handler = async move {
        while let Some(next) = ws_rx.next().await {
            let next = next?;
            if next.to_str() == Ok("request") {
                block_sender_notify.notify_one();
            } else if let Ok(batch) =
                bincode::deserialize::<schedule_util::BatchOutput>(next.as_bytes())
            {
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
    let result =
        futures::future::try_join4(forward_messages, block_request, input_handler, timeout).await;
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
