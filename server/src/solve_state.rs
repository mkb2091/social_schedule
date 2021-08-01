use crate::*;
use schedule_util::{Batch, BatchData, BatchId};
use std::collections::HashSet;

use futures::TryFutureExt;

pub struct ScheduleState {
    arg: Arc<schedule_util::ScheduleArg>,
    unclaimed: Mutex<Vec<(usize, Arc<Batch>)>>,
    clients: Mutex<HashSet<Arc<Client>>>,
    queue: Mutex<VecDeque<(Arc<Client>, OneShotSender)>>,
    next_block_id: AtomicU64,
    block_size: usize,
}

type OneShotSender = tokio::sync::oneshot::Sender<Arc<Batch>>;

impl ScheduleState {
    pub fn new(arg: Arc<schedule_util::ScheduleArg>) -> Self {
        let scheduler = schedule_solver::Scheduler::new(arg.get_tables(), arg.get_rounds());
        let block_size = scheduler.get_block_size();
        let mut init = vec![0; block_size];
        let _ = scheduler.initialise_buffer(&mut init);
        Self {
            arg,
            unclaimed: Mutex::new(vec![(
                0,
                Arc::new(Batch::new(BatchId::new(0), BatchData::new(init))),
            )]),
            clients: Mutex::new(HashSet::new()),
            queue: Mutex::new(Default::default()),
            next_block_id: AtomicU64::new(1),
            block_size,
        }
    }

    pub fn get_block(
        &self,
        client: &Arc<Client>,
    ) -> Result<
        Arc<schedule_util::Batch>,
        impl std::future::Future<Output = Result<Arc<schedule_util::Batch>, ApiError>>,
    > {
        {
            let mut clients = self.clients.lock().unwrap();
            if !clients.contains(client) {
                clients.insert(client.clone());
            }
        }
        if let Some((_, next)) = self.unclaimed.lock().unwrap().pop() {
            client.claim_block(next.clone());
            return Ok(next);
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.queue.lock().unwrap().push_back((client.clone(), tx));
        Err(rx.map_err(|_| ApiError::Completed))
    }

    fn add_single_block(&self, batch: Arc<Batch>) {
        if let Some((client, listener)) = self.queue.lock().unwrap().pop_front() {
            if listener.send(batch.clone()).is_ok() {
                client.claim_block(batch);
                return;
            }
        }
        let scheduler =
            schedule_solver::Scheduler::new(self.arg.get_tables(), self.arg.get_rounds());
        let mut unclaimed = self.unclaimed.lock().unwrap();
        let block = (
            scheduler.get_players_placed(&batch.get_data().get_ref()) as usize,
            batch,
        );
        let index = unclaimed
            .binary_search_by_key(&block.0, |(players_placed, _)| *players_placed)
            .unwrap_or_else(|index| index);
        unclaimed.insert(index, block);
    }

    pub fn add_batch_result(
        &self,
        client: &Arc<Client>,
        result: schedule_util::BatchOutputDeserialize,
    ) {
        if self.clients.lock().unwrap().contains(client) {
            if client
                .get_claimed()
                .lock()
                .unwrap()
                .remove(&result.get_base())
                .is_some()
            {
                for child in result.get_children() {
                    let id = self.next_block_id.fetch_add(1, Ordering::Relaxed);
                    self.add_single_block(Arc::new(Batch::new(
                        BatchId::new(id),
                        BatchData::new(child.collect()),
                    )));
                }
                // TODO: Handle notable
            } else {
                panic!("Invalid batch result");
            }
        } else {
            println!("Client not in list");
            // Client has been removed
        }
    }
    pub fn free_all_from_client(&self, client: &Arc<Client>) {
        if self.clients.lock().unwrap().remove(client) {
            for (_id, (data, _instant)) in client.get_claimed().lock().unwrap().drain() {
                self.add_single_block(data);
            }
            self.queue
                .lock()
                .unwrap()
                .retain(|(other_client, _)| other_client != client);
        }
    }

    pub fn get_unclaimed_len(&self) -> usize {
        self.unclaimed.lock().unwrap().len()
    }

    pub fn get_queue_len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn get_clients(&self) -> std::sync::MutexGuard<'_, HashSet<std::sync::Arc<Client>>> {
        self.clients.lock().unwrap()
    }

    pub fn get_counts(&self) -> (usize, usize, usize) {
        (
            self.unclaimed.lock().unwrap().len(),
            self.clients
                .lock()
                .unwrap()
                .iter()
                .map(|client| client.claimed_len())
                .sum(),
            self.queue.lock().unwrap().len(),
        )
    }
    pub fn get_block_size(&self) -> usize {
        self.block_size
    }
}
