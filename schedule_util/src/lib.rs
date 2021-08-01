#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

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

#[derive(Debug, Default, Copy, Clone)]
pub struct Stats {
    pub steps: u64,
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct BatchData {
    data: Box<[u64]>,
}

impl BatchData {
    pub fn new(data: Vec<u64>) -> Self {
        Self {
            data: data.into_boxed_slice(),
        }
    }
    pub fn get_ref(&self) -> &[u64] {
        &self.data
    }
}

#[derive(Debug, Default, Clone)]
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

pub struct InnerBlockIter<'a> {
    data: std::slice::ChunksExact<'a, u8>,
}

impl<'a> InnerBlockIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data: data.chunks_exact(8),
        }
    }
}

impl<'a> Iterator for InnerBlockIter<'a> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|chunk| {
            let mut array = [0; 8];
            array.copy_from_slice(chunk);
            u64::from_le_bytes(array)
        })
    }
}

pub struct BlockIter<'a> {
    data: std::slice::ChunksExact<'a, u8>,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = InnerBlockIter<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|chunk| InnerBlockIter::new(chunk))
    }
}

#[derive(Debug)]
pub struct ConvertError {}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConvertError {}

pub struct BatchSerialize<'a> {
    id: BatchId,
    data: &'a [u64],
}

impl<'a> BatchSerialize<'a> {
    pub fn new(id: BatchId, data: &'a [u64]) -> Self {
        Self { id, data }
    }
    pub fn get_size(&self) -> usize {
        16 + self.data.len() * 8
    }
    pub fn serialize(&self, buf: &mut [u8]) -> Result<(), ConvertError> {
        if buf.len() >= self.get_size() {
            buf[0..8].copy_from_slice(&self.id.id.to_le_bytes());
            buf[8..16].copy_from_slice(&(self.data.len() as u64).to_le_bytes());
            let offset = 16;
            for (i, value) in self.data.iter().enumerate() {
                buf[offset + i * 8..offset + (i + 1) * 8].copy_from_slice(&value.to_le_bytes());
            }
            Ok(())
        } else {
            Err(ConvertError {})
        }
    }
}

pub struct BatchDeserialize<'a> {
    id: BatchId,
    data: &'a [u8],
}

impl<'a> BatchDeserialize<'a> {
    pub fn deserialize(buf: &'a [u8]) -> Result<Self, ConvertError> {
        if buf.len() >= 16 {
            let mut array = [0; 8];
            array.copy_from_slice(&buf[0..8]);
            let id = u64::from_le_bytes(array);
            array.copy_from_slice(&buf[8..16]);
            let len = u64::from_le_bytes(array) as usize * 8;
            if buf.len() >= 16 + len {
                return Ok(Self {
                    id: BatchId::new(id),
                    data: &buf[16..16 + len],
                });
            }
        }
        Err(ConvertError {})
    }

    pub fn get_id(&self) -> BatchId {
        self.id
    }

    pub fn get_length(&self) -> usize {
        self.data.len() / 8
    }

    pub fn get_data(&self) -> InnerBlockIter<'a> {
        InnerBlockIter::new(self.data)
    }
}

#[derive(Debug)]
pub struct BatchOutputSerialize<'a> {
    base: BatchId,
    block_size: usize,
    children: &'a [u64],
    notable: &'a [u64],
    stats: Stats,
}

impl<'a> BatchOutputSerialize<'a> {
    pub fn new(
        base: BatchId,
        block_size: usize,
        children: &'a [u64],
        notable: &'a [u64],
        stats: Stats,
    ) -> Self {
        Self {
            base,
            block_size,
            children,
            notable,
            stats,
        }
    }
    pub fn get_size(&self) -> usize {
        44 + (self.children.len() + self.notable.len()) * 8
    }

    pub fn serialize(&self, buf: &mut [u8]) -> Result<(), ConvertError> {
        if buf.len() >= self.get_size() {
            buf[0..8].copy_from_slice(&self.base.id.to_le_bytes());
            buf[8..16].copy_from_slice(&(self.children.len() as u64).to_le_bytes());
            buf[16..24].copy_from_slice(&(self.notable.len() as u64).to_le_bytes());
            buf[24..32].copy_from_slice(&self.stats.steps.to_le_bytes());
            buf[32..40].copy_from_slice(&self.stats.elapsed.as_secs().to_le_bytes());
            buf[40..44].copy_from_slice(&self.stats.elapsed.subsec_nanos().to_le_bytes());
            let offset = 44;
            for (i, val) in self.children.iter().enumerate() {
                buf[i * 8 + offset..(i + 1) * 8 + offset].copy_from_slice(&val.to_le_bytes());
            }
            let offset = offset + self.children.len() * 8;
            for (i, val) in self.notable.iter().enumerate() {
                buf[i * 8 + offset..(i + 1) * 8 + offset].copy_from_slice(&val.to_le_bytes());
            }
            Ok(())
        } else {
            Err(ConvertError {})
        }
    }
}

#[derive(Debug, Default)]
pub struct BatchOutputDeserialize<'a> {
    base: BatchId,
    block_size: usize,
    children: &'a [u8],
    notable: &'a [u8],
    stats: Stats,
}

impl<'a> BatchOutputDeserialize<'a> {
    pub fn deserialize(block_size: usize, data: &'a [u8]) -> Result<Self, ConvertError> {
        if data.len() < 44 {
            return Err(ConvertError {});
        }
        let mut array = [0; 8];
        array.copy_from_slice(&data[0..8]);
        let id = u64::from_le_bytes(array);
        array.copy_from_slice(&data[8..16]);
        let children_length = u64::from_le_bytes(array);
        array.copy_from_slice(&data[16..24]);
        let notable_length = u64::from_le_bytes(array);
        array.copy_from_slice(&data[24..32]);
        let steps = u64::from_le_bytes(array);
        array.copy_from_slice(&data[32..40]);
        let secs = u64::from_le_bytes(array);
        let mut array = [0; 4];
        array.copy_from_slice(&data[40..44]);
        let nanos = u32::from_le_bytes(array);
        let total_length = ((children_length + notable_length) as u64 * 8 + 44) as usize;
        if total_length > data.len() {
            return Err(ConvertError {});
        }
        let (children, notable) = data[44..total_length].split_at(children_length as usize * 8);
        Ok(Self {
            base: BatchId::new(id),
            block_size,
            children,
            notable,
            stats: Stats {
                steps: steps,
                elapsed: std::time::Duration::new(secs, nanos),
            },
        })
    }
    pub fn get_base(&self) -> BatchId {
        self.base
    }
    pub fn get_children(&self) -> BlockIter<'a> {
        BlockIter {
            data: self.children.chunks_exact(self.block_size * 8),
        }
    }
    pub fn get_notable(&self) -> BlockIter<'a> {
        BlockIter {
            data: self.notable.chunks_exact(self.block_size * 8),
        }
    }
    pub fn get_stats(&self) -> Stats {
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_iter() {
        let data = vec![0; 8];
        assert_eq!(
            BlockIter {
                data: data.chunks_exact(8)
            }
            .flatten()
            .collect::<Vec<_>>(),
            vec![0]
        );
    }
    #[quickcheck]
    fn deserialize_does_not_panic(block_size: std::num::NonZeroUsize, data: Vec<u8>) {
        let _ = BatchOutputDeserialize::deserialize(block_size.get(), &data);
    }

    #[quickcheck]
    fn decodes_to_self(
        id: u64,
        block_size: std::num::NonZeroUsize,
        children: Vec<u64>,
        notable: Vec<u64>,
        steps: u64,
        elapsed: std::time::Duration,
    ) {
        let block_size = block_size.get();
        let children = &children[..children.len() / block_size * block_size];
        let notable = &notable[..notable.len() / block_size * block_size];
        let s = BatchOutputSerialize::new(
            BatchId::new(id),
            block_size,
            children,
            notable,
            Stats { steps, elapsed },
        );
        let mut buf = vec![0; s.get_size()];
        s.serialize(&mut buf).unwrap();
        let deserialize = BatchOutputDeserialize::deserialize(block_size, &buf).unwrap();
        println!("s: {:?}\nd: {:?}\n", s, deserialize);
        assert_eq!(id, deserialize.get_base().id);
        assert_eq!(steps, deserialize.get_stats().steps);
        assert_eq!(elapsed, deserialize.get_stats().elapsed);
        assert_eq!(id, deserialize.get_base().id);
        assert_eq!(
            &children,
            &deserialize.get_children().flatten().collect::<Vec<_>>()
        );
        //assert_eq!(&notable, &deserialize.get_children().flatten().collect::<Vec<_>>());
    }
}
