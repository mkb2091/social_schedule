#[derive(
    Clone, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Debug,
)]
pub struct ScheduleArg {
    tables: Box<[usize]>,
    rounds: usize,
}

impl ScheduleArg {
    pub fn new(tables: &[usize], rounds: usize) -> Self {
        Self {
            tables: tables.to_vec().into_boxed_slice(),
            rounds,
        }
    }
    pub fn get_tables(&self) -> &[usize] {
        &self.tables
    }
    pub fn get_rounds(&self) -> usize {
        self.rounds
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Stats {
    pub steps: usize,
    pub elapsed: std::time::Duration,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct BatchOutput {
    pub base: Vec<u64>,
    pub children: Vec<Vec<u64>>,
    pub notable: Vec<Vec<u64>>,
    pub stats: Stats,
}
