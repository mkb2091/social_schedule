pub mod api;
pub mod ui_pages;

use std::collections::{HashMap, VecDeque};
use std::sync::{atomic::*, Arc, Mutex};

pub use seed;
pub use warp;

#[derive(Debug)]
pub struct Completed {}

impl std::fmt::Display for Completed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Completed")
    }
}

impl std::error::Error for Completed {}

pub struct ScheduleState {
    arg: Arc<schedule_util::ScheduleArg>,
    unclaimed: Mutex<Vec<Vec<usize>>>,
    claimed: Mutex<HashMap<Vec<usize>, (schedule_util::ClientId, std::time::Instant)>>,
    queue: Mutex<VecDeque<(schedule_util::ClientId, OneShotSender)>>,
}

type OneShotSender = tokio::sync::oneshot::Sender<Vec<usize>>;

impl ScheduleState {
    pub fn new(arg: Arc<schedule_util::ScheduleArg>) -> Self {
        let scheduler = schedule_solver::Scheduler::new(arg.get_tables(), arg.get_rounds());
        let mut init = vec![0; scheduler.get_block_size()];
        let _ = scheduler.initialise_buffer(&mut init);
        Self {
            arg,
            unclaimed: Mutex::new(vec![init]),
            claimed: Mutex::new(Default::default()),
            queue: Mutex::new(Default::default()),
        }
    }

    pub async fn get_block(
        &self,
        client_id: schedule_util::ClientId,
    ) -> Result<Vec<usize>, Completed> {
        {
            let mut unclaimed = self.unclaimed.lock().unwrap();
            let mut claimed = self.claimed.lock().unwrap();
            if let Some(next) = unclaimed.pop() {
                claimed.insert(next.clone(), (client_id, std::time::Instant::now()));
                return Ok(next);
            }
            if claimed.is_empty() {
                return Err(Completed {});
            }
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.queue.lock().unwrap().push_back((client_id, tx));
        rx.await.map_err(|_| Completed {})
    }

    pub fn get_counts(&self) -> (usize, usize, usize) {
        (
            self.unclaimed.lock().unwrap().len(),
            self.claimed.lock().unwrap().len(),
            self.queue.lock().unwrap().len(),
        )
    }

    fn add_single_block(&self, block: Vec<usize>) {
        if let Some((client_id, listener)) = self.queue.lock().unwrap().pop_front() {
            if listener.send(block.clone()).is_ok() {
                self.claimed
                    .lock()
                    .unwrap()
                    .insert(block, (client_id, std::time::Instant::now()));
                return;
            }
        }
        let scheduler =
            schedule_solver::Scheduler::new(self.arg.get_tables(), self.arg.get_rounds());
        let mut unclaimed = self.unclaimed.lock().unwrap();
        unclaimed.push(block);
        unclaimed.sort_unstable_by_key(|block| scheduler.get_players_placed(block));
    }

    pub fn add_batch_result(&self, result: schedule_util::BatchOutput) {
        if self.claimed.lock().unwrap().remove(&result.base).is_some() {
            if result.children.len() == 0
                && self.claimed.lock().unwrap().is_empty()
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
    }
    pub fn free_all_from_client(&self, client_id: schedule_util::ClientId) {
        let mut unclaimed = self.unclaimed.lock().unwrap();
        let mut claimed = self.claimed.lock().unwrap();
        claimed.retain(|block, (other_id, _)| {
            if *other_id == client_id {
                unclaimed.push(block.to_vec());
                false
            } else {
                true
            }
        });
    }
}

pub struct State {
    pub scheduler: Mutex<(Vec<usize>, usize)>,
    schedule_solve_states: Mutex<HashMap<Arc<schedule_util::ScheduleArg>, Arc<ScheduleState>>>,
    pub next_client_id: AtomicUsize,
}

impl State {
    pub fn new() -> Self {
        let scheduler = Mutex::new((vec![], 0));
        let schedule_solve_states = Mutex::new(HashMap::new());
        let next_client_id = AtomicUsize::new(0);
        State {
            scheduler,
            schedule_solve_states,
            next_client_id,
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
