pub mod api;
pub mod solve_state;
pub mod ui_pages;

use schedule_util::{Batch, BatchData, BatchId};
pub use solve_state::ScheduleState;

use std::collections::{HashMap, VecDeque};
use std::sync::{atomic::*, Arc, Mutex};

pub use seed;
pub use warp;

#[derive(Debug)]
pub enum ApiError {
    Completed,
    StreamFinished,
    Timeout,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ApiError {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClientId {
    id: usize,
}

impl ClientId {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientID({})", self.id)
    }
}

#[derive(Debug)]
struct RateStats {
    data: [(std::time::Instant, usize); 10],
}

impl RateStats {
    fn new() -> Self {
        Self {
            data: [(std::time::Instant::now(), 0); 10],
        }
    }
    fn add(&mut self, amount: usize) {
        if self.data[0].0.elapsed().as_secs() >= 1 {
            self.data.rotate_right(1);
            self.data[0] = (std::time::Instant::now(), 0);
        }
        for (_, counter) in self.data.iter_mut() {
            *counter += amount;
        }
    }
    fn rate(&self) -> f32 {
        let (start, amount) = self.data.last().unwrap();
        *amount as f32 / start.elapsed().as_secs_f32()
    }
}

#[derive(Debug)]
pub struct Client {
    id: ClientId,
    last_message: Mutex<std::time::Instant>,
    claimed: Mutex<HashMap<BatchId, BatchData>>,
    step_counts: Mutex<RateStats>,
    data_sent: Mutex<RateStats>,
    data_recieved: Mutex<RateStats>,
}

impl std::cmp::PartialEq for Client {
    fn eq(&self, rhs: &Self) -> bool {
        self.id.eq(&rhs.id)
    }
}

impl std::cmp::Eq for Client {}

impl std::hash::Hash for Client {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Client {
    pub fn new(id: usize) -> Self {
        Self {
            id: ClientId::new(id),
            last_message: Mutex::new(std::time::Instant::now()),
            claimed: Mutex::new(HashMap::new()),
            step_counts: Mutex::new(RateStats::new()),
            data_sent: Mutex::new(RateStats::new()),
            data_recieved: Mutex::new(RateStats::new()),
        }
    }
    pub fn get_last_updated(&self) -> std::time::Instant {
        *self.last_message.lock().unwrap()
    }
    pub fn get_id(&self) -> ClientId {
        self.id
    }
    pub fn claimed_len(&self) -> usize {
        self.claimed.lock().unwrap().len()
    }
    pub fn get_claimed(&self) -> &Mutex<HashMap<BatchId, BatchData>> {
        &self.claimed
    }
    pub fn claim_block(&self, batch: Batch) {
        let (id, data) = batch.split();
        self.claimed.lock().unwrap().insert(id, data);
        *self.last_message.lock().unwrap() = std::time::Instant::now();
    }
    pub fn add_stats(&self, stats: &schedule_util::Stats) {
        self.step_counts.lock().unwrap().add(stats.steps);
    }

    pub fn get_rate(&self) -> f32 {
        self.step_counts.lock().unwrap().rate()
    }

    pub fn add_recieved_bytes(&self, amount: usize) {
        self.data_recieved.lock().unwrap().add(amount)
    }

    pub fn add_sent_bytes(&self, amount: usize) {
        self.data_sent.lock().unwrap().add(amount)
    }

    pub fn get_recieved_rate(&self) -> f32 {
        self.data_recieved.lock().unwrap().rate()
    }
    pub fn get_sent_rate(&self) -> f32 {
        self.data_sent.lock().unwrap().rate()
    }
}

pub struct State {
    pub scheduler: Mutex<(Vec<usize>, usize)>,
    schedule_solve_states: Mutex<HashMap<Arc<schedule_util::ScheduleArg>, Arc<ScheduleState>>>,
    pub next_client_id: AtomicUsize,
    pub client_buffer_size: AtomicUsize,
    pub timeout: AtomicU64,
}

impl State {
    pub fn new() -> Self {
        let scheduler = Mutex::new((vec![], 0));
        let schedule_solve_states = Mutex::new(HashMap::new());
        let next_client_id = AtomicUsize::new(0);
        let client_buffer_size = AtomicUsize::new(100);
        let timeout = AtomicU64::new(10);
        State {
            scheduler,
            schedule_solve_states,
            next_client_id,
            client_buffer_size,
            timeout,
        }
    }

    pub fn get_schedule_solve_state(
        &self,
        arg: Arc<schedule_util::ScheduleArg>,
    ) -> Arc<ScheduleState> {
        self.schedule_solve_states
            .lock()
            .unwrap()
            .entry(arg.clone())
            .or_insert_with(|| Arc::new(ScheduleState::new(arg)))
            .clone()
    }

    pub fn all_schedule_solve_states(
        &self,
    ) -> HashMap<Arc<schedule_util::ScheduleArg>, Arc<ScheduleState>> {
        self.schedule_solve_states.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn get_block_return_first() {
        let arg = Arc::new(schedule_util::ScheduleArg::new(&[4; 6], 4));
        let solve_state = State::new().get_schedule_solve_state(arg);
        let id = schedule_util::ClientId::new(0);
        let first = solve_state.get_block(id).await.unwrap();
        assert!(solve_state.unclaimed.lock().unwrap().is_empty());
        assert_eq!(solve_state.claimed.lock().unwrap().len(), 1);
        let mut batch_result = schedule_util::BatchOutput::default();
        batch_result.base = first;
        solve_state.add_batch_result(batch_result);
        assert!(solve_state.unclaimed.lock().unwrap().is_empty());
        assert!(solve_state.claimed.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn get_block_concurrent() {
        let arg = Arc::new(schedule_util::ScheduleArg::new(&[4; 6], 4));
        let solve_state = State::new().get_schedule_solve_state(arg);
        let id = schedule_util::ClientId::new(0);
        let first = solve_state.get_block(id).await.unwrap();
        let mut tasks = Vec::new();
        for i in 1..5 {
            let solve_state = solve_state.clone();
            tasks.push(tokio::spawn(async move {
                let id = schedule_util::ClientId::new(i);
                let block = solve_state.get_block(id).await.unwrap();
                let mut batch_result = schedule_util::BatchOutput::default();
                batch_result.base = block.clone();
                batch_result.children = vec![block];
                solve_state.add_batch_result(batch_result);
            }));
        }
        assert!(solve_state.unclaimed.lock().unwrap().is_empty());
        assert_eq!(solve_state.claimed.lock().unwrap().len(), 1);
        let mut batch_result = schedule_util::BatchOutput::default();
        batch_result.base = first.clone();
        batch_result.children = vec![first];
        solve_state.add_batch_result(batch_result);
        for task in tasks.into_iter() {
            task.await;
        }

        assert_eq!(solve_state.unclaimed.lock().unwrap().len(), 1);
        assert_eq!(solve_state.claimed.lock().unwrap().len(), 0);
        assert_eq!(solve_state.queue.lock().unwrap().len(), 0);
    }
}
