use crate::*;
use std::collections::HashSet;

pub struct ScheduleState {
    arg: Arc<schedule_util::ScheduleArg>,
    unclaimed: Mutex<Vec<(usize, Vec<u64>)>>,
    clients: Mutex<HashSet<Arc<Client>>>,
    queue: Mutex<VecDeque<(Arc<Client>, OneShotSender)>>,
}

type OneShotSender = tokio::sync::oneshot::Sender<Vec<u64>>;

impl ScheduleState {
    pub fn new(arg: Arc<schedule_util::ScheduleArg>) -> Self {
        let scheduler = schedule_solver::Scheduler::new(arg.get_tables(), arg.get_rounds());
        let mut init = vec![0; scheduler.get_block_size()];
        let _ = scheduler.initialise_buffer(&mut init);
        Self {
            arg,
            unclaimed: Mutex::new(vec![(0, init)]),
            clients: Mutex::new(HashSet::new()),
            queue: Mutex::new(Default::default()),
        }
    }

    pub async fn get_block(&self, client: &Arc<Client>) -> Result<Vec<u64>, ApiError> {
        {
            let mut clients = self.clients.lock().unwrap();
            if !clients.contains(client) {
                clients.insert(client.clone());
            }
        }
        {
            let mut unclaimed = self.unclaimed.lock().unwrap();
            if let Some((_, next)) = unclaimed.pop() {
                client.claim_block(next.clone());
                return Ok(next);
            }
            if self.clients.lock().unwrap().is_empty() {
                return Err(ApiError::Completed);
            }
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.queue.lock().unwrap().push_back((client.clone(), tx));
        rx.await.map_err(|_| ApiError::Completed)
    }

    fn add_single_block(&self, block: Vec<u64>) {
        if let Some((client, listener)) = self.queue.lock().unwrap().pop_front() {
            if listener.send(block.clone()).is_ok() {
                client.claim_block(block);
                return;
            }
        }
        let scheduler =
            schedule_solver::Scheduler::new(self.arg.get_tables(), self.arg.get_rounds());
        let mut unclaimed = self.unclaimed.lock().unwrap();
        unclaimed.push((scheduler.get_players_placed(&block) as usize, block));
        unclaimed.sort_unstable_by_key(|(players_placed, _block)| *players_placed);
    }

    pub fn add_batch_result(&self, client: &Arc<Client>, result: schedule_util::BatchOutput) {
        if self.clients.lock().unwrap().contains(client) {
            if client.get_claimed().lock().unwrap().remove(&result.base) {
                if result.children.len() == 0
                    && self.clients.lock().unwrap().is_empty()
                    && self.unclaimed.lock().unwrap().is_empty()
                {
                    self.queue.lock().unwrap().clear();
                }
                for child in result.children.into_iter() {
                    if &child != &result.base {
                        self.add_single_block(child);
                    } else {
                        panic!();
                    }
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
            for block in client.get_claimed().lock().unwrap().drain() {
                self.add_single_block(block);
            }
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
}
