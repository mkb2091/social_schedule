use crate::*;
use std::sync::{atomic::Ordering, Arc};
use warp::ws::{Message, WebSocket};
use warp::{filters::BoxedFilter, Filter, Reply};

use futures::pin_mut;
use futures::stream::{SplitSink, SplitStream};
use futures::FutureExt;
use futures::SinkExt;
use futures::StreamExt;

use schedule_util::BatchSerialize;

async fn send_blocks(
    client: Arc<Client>,
    state: Arc<State>,
    solve_state: Arc<ScheduleState>,
    mut ws_tx: SplitSink<warp::ws::WebSocket, warp::ws::Message>,
    notify: Arc<tokio::sync::Notify>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        notify.notified().await;
        let client_buffer_size = state.client_buffer_size.load(Ordering::Relaxed);
        let mut amount = client_buffer_size.saturating_sub(client.claimed_len());
        let mut sent = false;
        let mut i = 0;
        while i < amount {
            i += 1;
            let block = match solve_state.get_block(&client) {
                Ok(block) => block,
                Err(fut) => {
                    if sent {
                        // Don't flush for first block
                        ws_tx.flush().await?;
                    }
                    let result = fut.await?;
                    i = 0;
                    amount = client_buffer_size.saturating_sub(client.claimed_len());
                    result
                }
            };
            let block: &Batch = &block;
            let serialized = BatchSerialize::new(block.get_id(), &block.get_data().get_ref());
            let mut buf = Vec::with_capacity(serialized.get_size());
            buf.resize(serialized.get_size(), 0);
            serialized.serialize(&mut buf)?;
            client.add_sent_bytes(buf.len());
            ws_tx.feed(Message::binary(buf)).await?;
            sent = true;
        }
        if sent {
            ws_tx.flush().await?;
        }
    }
}

async fn input_handler(
    client: Arc<Client>,
    solve_state: Arc<ScheduleState>,
    mut ws_rx: SplitStream<warp::ws::WebSocket>,
    block_sender_notify: Arc<tokio::sync::Notify>,
) -> Result<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let next = ws_rx.next().await.ok_or(ApiError::StreamFinished)??;
        let next = next.as_bytes();
        client.add_recieved_bytes(next.len());
        if let Ok(batch) =
            schedule_util::BatchOutputDeserialize::deserialize(solve_state.get_block_size(), &next)
        {
            client.add_stats(&batch.get_stats());
            solve_state.add_batch_result(&client, batch);
            block_sender_notify.notify_one();
        } else {
            println!("Unknown data: {:?}", next);
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
    let (ws_tx, ws_rx) = ws.split();
    let block_sender_notify = Arc::new(tokio::sync::Notify::new());
    let block_request = tokio::spawn(send_blocks(
        client.clone(),
        state.clone(),
        solve_state.clone(),
        ws_tx,
        block_sender_notify.clone(),
    ));
    block_sender_notify.notify_one();
    let input_handler = tokio::spawn(input_handler(
        client.clone(),
        solve_state.clone(),
        ws_rx,
        block_sender_notify.clone(),
    ));
    let client_clone = client.clone();
    let timeout = async move {
        loop {
            let timeout = state.timeout.load(Ordering::Relaxed);
            let timeout = std::time::Duration::from_secs(timeout);
            tokio::time::sleep(timeout).await;
            if client_clone.get_last_updated().elapsed() > timeout {
                return Err::<std::convert::Infallible, Box<dyn std::error::Error + Send + Sync>>(
                    Box::new(ApiError::Timeout),
                );
            }
        }
    };
    pin_mut!(block_request);
    pin_mut!(input_handler);
    let result = futures::future::select(&mut block_request, &mut input_handler)
        .map(|either| either.factor_first().0)
        .map(|result| {
            let result = result.map_err(|join_err| {
                let err: Box<dyn std::error::Error + Send + Sync> = Box::new(join_err);
                err
            });
            result.unwrap_or_else(|err| Err(err))
        });
    pin_mut!(timeout);
    let result = futures::future::select(result, timeout)
        .map(|either| either.factor_first().0)
        .await;
    block_request.abort();
    input_handler.abort();
    println!("Client {} Result: {:?}", client.get_id(), result);
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
