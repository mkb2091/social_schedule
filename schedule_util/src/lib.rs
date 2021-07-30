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

#[derive(
    serde::Deserialize, serde::Serialize, Copy, Clone, Debug, Default, Eq, PartialEq, Hash,
)]
pub struct BatchId {
    id: u64,
}

impl BatchId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct BatchData {
    data: Vec<u64>,
}

impl BatchData {
    pub fn new(data: Vec<u64>) -> Self {
        Self { data }
    }
    pub fn get_ref(&self) -> &'_ Vec<u64> {
        &self.data
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone)]
pub struct Batch {
    id: BatchId,
    data: BatchData,
}

impl Batch {
    pub fn new(id: BatchId, data: BatchData) -> Self {
        Self { id, data }
    }
    pub fn split(self) -> (BatchId, BatchData) {
        (self.id, self.data)
    }
    pub fn get_id(&self) -> BatchId {
        self.id
    }
    pub fn get_data(&self) -> &'_ BatchData {
        &self.data
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct BatchOutput {
    pub base: BatchId,
    pub children: Vec<Vec<u64>>,
    pub notable: Vec<Vec<u64>>,
    pub stats: Stats,
}
