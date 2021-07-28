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

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ClientId {
    id: usize,
}

impl ClientId {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct Stats {
    pub steps: usize,
    pub elapsed: std::time::Duration,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct BatchOutput {
    pub base: Vec<usize>,
    pub children: Vec<Vec<usize>>,
    pub notable: Vec<Vec<usize>>,
    pub stats: Stats,
}
