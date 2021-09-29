use crate::util::*;
use crate::word::Word;
use crate::word::*;

#[derive(Debug)]
pub enum SchedulerErrors {
    ZeroLengthGroups,
    PlayerCountOverflow,
    TooSmallBuffer,
    RoundsTooLarge,
}

pub struct Sizes {
    player_bit_word_count: usize,
    to_explore_size: usize,
    played_with_size: usize,
    played_on_table_size: usize,
    played_in_round_size: usize,
    played_in_game_size: usize,
}

impl Sizes {
    const fn new(tables: &[usize], rounds: usize, t_bit_count: usize) -> Self {
        let mut player_count: usize = 0;
        let mut i = 0;
        while i < tables.len() {
            player_count += tables[i];
            i += 1;
        }
        let player_bit_word_count =
            player_count / t_bit_count + (player_count % t_bit_count != 0) as usize;
        let played_with_size = player_bit_word_count * player_count;
        let played_on_table_size = player_bit_word_count * tables.len();
        let played_in_round_size = player_bit_word_count * rounds;
        let played_in_game_size = player_bit_word_count * tables.len() * rounds;
        let mut to_explore_size = rounds * tables.len() * 2;
        to_explore_size =
            to_explore_size / t_bit_count + (to_explore_size % t_bit_count != 0) as usize;
        Self {
            player_bit_word_count,
            to_explore_size,
            played_with_size,
            played_on_table_size,
            played_in_round_size,
            played_in_game_size,
        }
    }
    pub const fn get_player_bit_word_count(&self) -> usize {
        self.player_bit_word_count
    }
    pub const fn get_to_explore_size(&self) -> usize {
        self.to_explore_size
    }
    pub const fn get_played_with_size(&self) -> usize {
        self.played_with_size
    }
    pub const fn get_played_on_table_size(&self) -> usize {
        self.played_on_table_size
    }
    pub const fn get_played_in_round_size(&self) -> usize {
        self.played_in_round_size
    }
    pub const fn get_played_in_game_size(&self) -> usize {
        self.played_in_game_size
    }
    pub const fn get_potential_in_game_size(&self) -> usize {
        self.played_in_game_size
    }
    pub const fn get_total_size(&self) -> usize {
        self.to_explore_size
            + self.played_with_size
            + self.played_on_table_size
            + self.played_in_round_size
            + self.played_in_game_size
    }
}
pub const fn bit_length<T>() -> usize {
    core::mem::size_of::<T>() * 8
}

const fn get_byte_and_mask<T>(bit: usize) -> (usize, u64) {
    let byte = bit / bit_length::<T>();
    let mask = 1 << (bit - (byte * bit_length::<T>()));
    (byte, mask)
}

trait Block<'a, T: Word>:
    std::ops::Index<usize, Output = T> + std::ops::IndexMut<usize, Output = T>
{
    fn len(&self) -> usize;
    fn fill(&mut self);
    fn clear(&mut self);
}

struct Single<'a, T: Word> {
    block: &'a mut T,
}

impl<'a, T: Word> std::ops::Index<usize> for Single<'a, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.block
    }
}

impl<'a, T: Word> std::ops::IndexMut<usize> for Single<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.block
    }
}

impl<'a, T: Word> Block<'a, T> for Single<'a, T> {
    fn len(&self) -> usize {
        0
    }
    fn fill(&mut self) {
        *self.block = T::ONE;
    }

    fn clear(&mut self) {
        *self.block = T::ZERO;
    }
}

struct Multiple<'a, T: Word> {
    block: &'a mut [T],
}

impl<'a, T: Word> std::ops::Index<usize> for Multiple<'a, T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.block.index(index)
    }
}

impl<'a, T: Word> std::ops::IndexMut<usize> for Multiple<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.block.index_mut(index)
    }
}

impl<'a, T: Word> Block<'a, T> for Multiple<'a, T> {
    fn len(&self) -> usize {
        self.block.len()
    }
    fn fill(&mut self) {
        self.block.fill(T::ONE);
    }

    fn clear(&mut self) {
        self.block.fill(T::ZERO);
    }
}

impl<'a, T: Word> Multiple<'a, T> {
    fn new(data: &'a mut [T]) -> Self {
        Self { block: data }
    }
}

struct ToExplore<'a, T> {
    data: &'a mut [T],
    divisor: u32,
}

impl<'a, T> ToExplore<'a, T> {
    const fn calc_divisor(tables: usize) -> u32 {
        tables.next_power_of_two().trailing_zeros()
    }
}

impl<'a, T: Word> ToExplore<'a, T> {
    fn new(block: &'a mut [T], tables: usize) -> Self {
        let divisor = Self::calc_divisor(tables);
        Self::new_with_power(block, divisor)
    }

    fn new_with_power(block: &'a mut [T], divisor: u32) -> Self {
        Self {
            data: block,
            divisor,
        }
    }

    fn copy(&'a mut self) -> Self {
        Self {
            data: self.data,
            divisor: self.divisor,
        }
    }

    fn get_data(&'a mut self) -> &'a mut [T] {
        self.data
    }

    fn set(&mut self, round: usize, table: usize, state: bool) {
        let number = (round << self.divisor as usize) + table;
        let index = number / bit_length::<T>();
        let mask = T::ONE << (number - (index * bit_length::<T>()));
        if state {
            self.data[index] |= mask;
        } else {
            self.data[index] ^= mask;
        }
    }

    fn iter_mut(&'a mut self) -> ToExploreIter<'a, T> {
        ToExploreIter::new(self)
    }
}

struct ToExploreIter<'a, T: Word> {
    to_explore: ToExplore<'a, T>,
    current_block: Option<T>,
    index: usize,
}

impl<'a, T: Word> Iterator for ToExploreIter<'a, T> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current_block.as_mut()?;
        while *current == T::ZERO {
            self.index += 1;
            *current = *self.to_explore.data.get(self.index)?;
        }

        let trailing_zeros = current.trailing_zeros();
        *current ^= T::ONE << trailing_zeros as usize;
        let number = self.index * bit_length::<T>() + trailing_zeros as usize;
        let round = number >> self.to_explore.divisor as usize;
        let table = number ^ (round << self.to_explore.divisor as usize);

        return Some((round, table));

        None
    }
}

impl<'a, T: Word> ToExploreIter<'a, T> {
    fn new(to_explore: &'a mut ToExplore<'a, T>) -> Self {
        Self {
            current_block: Some(to_explore.data[0]),
            to_explore: to_explore.copy(),
            index: 0,
        }
    }

    fn remove(&mut self, round: usize, table: usize) {
        self.to_explore.set(round, table, false);
    }

    fn get_data(&'a mut self) -> &'a mut [T] {
        self.to_explore.get_data()
    }
}

#[test]
fn test_to_explore_words() {
    let mut data = vec![0_u8; 16];
    let mut to_explore = ToExplore::new(&mut data, 8);
    for round in 0..8 {
        for table in 0..8 {
            to_explore.set(round, table, true);
        }
    }
    let iter = &mut to_explore.iter_mut();
    println!("data: {:?}", iter.to_explore.data);
    let result = iter.take(3).collect::<Vec<_>>();
    println!("data: {:?}", iter.to_explore.data);
    assert_eq!(result, vec![(0, 0), (0, 1), (0, 2)]);
    let result = iter.skip(4).take(4).collect::<Vec<_>>();
    assert_eq!(result, vec![(0, 7), (1, 0), (1, 1), (1, 2)]);
}

pub trait ScheduleT<T: Word> {
    fn new(sizes: Sizes) -> Option<Self>
    where
        Self: Sized;
    fn get_to_explore(&self) -> &[T];
    fn get_to_explore_mut(&mut self) -> &mut [T];
    fn get_played_with(&self) -> &[T];
    fn get_played_with_mut(&mut self) -> &mut [T];
    fn get_played_on_table(&self) -> &[T];
    fn get_played_on_table_mut(&mut self) -> &mut [T];
    fn get_played_in_round(&self) -> &[T];
    fn get_played_in_round_mut(&mut self) -> &mut [T];
    fn get_played_in_game(&self) -> &[T];
    fn get_played_in_game_mut(&mut self) -> &mut [T];
    fn get_potential_in_game(&self) -> &[T];
    fn get_potential_in_game_mut(&mut self) -> &mut [T];
}

pub trait SchedulerTrait {
    type S;
}

pub struct Scheduler2 {
    sizes: Sizes,
}

impl Scheduler2 {
    const fn new() {}
}

struct DynSchedule {
    a: Box<dyn ScheduleT<usize>>,
}

#[derive(Debug)]
pub enum SchedulerResult {}

pub struct Schedule<'a> {
    to_explore: ToExplore<'a, u64>,
    rest: &'a mut [u64],
}

impl<'a> Schedule<'a> {
    pub fn import_buffer<'b>(buffer: &'a mut [u64], scheduler: &Scheduler<'b>) -> Option<Self> {
        if buffer.len() < scheduler.get_block_size() {
            None
        } else {
            let (to_explore, buffer) = buffer.split_at_mut(scheduler.offsets.to_explore_size);
            let to_explore = ToExplore::new(to_explore, scheduler.offsets.divisor);
            Some(Schedule {
                to_explore,
                rest: buffer,
            })
        }
    }

    pub fn copy(&'a mut self) -> Self {
        Self {
            to_explore: self.to_explore.copy(),
            rest: self.rest,
        }
    }

    fn extract_to_explore(&'a mut self) -> (&'a mut Self, ToExplore<'a, u64>) {
        let to_explore = std::mem::replace(&mut self.to_explore, ToExplore::new(&mut [], 0));
        (self, to_explore)
    }

    fn import_to_explore(&'a mut self, to_explore: ToExplore<'a, u64>) {
        self.to_explore = to_explore;
    }
}

#[derive(Debug)]
struct Offsets {
    players_placed_counter_offset: usize,
    empty_table_count_offset: usize,
    to_explore_offset: usize,
    to_explore_size: usize,
    divisor: usize,
    played_with_offset: usize,
    played_on_table_total_offset: usize,
    played_in_round_offset: usize,
    played_on_table_offset: usize,
    potential_on_table_offset: usize,
    played_on_table_size: usize,
    block_size: usize,
}

impl Offsets {
    const fn new(
        to_explore_size: usize,
        divisor: usize,
        played_with_size: usize,
        played_on_table_total_size: usize,
        played_in_round_size: usize,
        played_on_table_size: usize,
    ) -> Self {
        let to_explore_offset = 0;
        let players_placed_counter_offset = 0;
        let empty_table_count_offset = players_placed_counter_offset + 1;
        let played_with_offset = empty_table_count_offset + 1;
        let played_on_table_total_offset = played_with_offset + played_with_size;
        let played_in_round_offset = played_on_table_total_offset + played_on_table_total_size;
        let played_on_table_offset = played_in_round_offset + played_in_round_size;
        let potential_on_table_offset = played_on_table_offset + played_on_table_size;
        let block_size = potential_on_table_offset + played_on_table_size + to_explore_size;
        Self {
            players_placed_counter_offset,
            empty_table_count_offset,
            to_explore_offset,
            to_explore_size,
            divisor,
            played_with_offset,
            played_on_table_total_offset,
            played_in_round_offset,
            played_on_table_offset,
            potential_on_table_offset,
            played_on_table_size,
            block_size,
        }
    }
    const fn potential_on_table(&self) -> usize {
        self.potential_on_table_offset
    }
}
const ROUND_COUNT: usize = 6;
const TABLE_COUNT: usize = 6;
const PLAYERS_PER_TABLE: usize = 4;
const TO_EXPLORE_SHIFT: usize = 3;
const PLAYER_COUNT: usize = TABLE_COUNT * PLAYERS_PER_TABLE;

#[derive(Copy, Clone, Debug)]
pub struct State {
    tables_to_explore: u64,
    players_played_count: u8,
    empty_table_count: u8,
    players_played_with: [u32; PLAYER_COUNT],
    played_in_round: [u32; ROUND_COUNT],
    played_on_table: [u32; ROUND_COUNT * TABLE_COUNT],
    potential_on_table: [u32; ROUND_COUNT * TABLE_COUNT],
    played_on_table_total: [u32; TABLE_COUNT],
}

impl State {
    pub fn new() -> Self {
        let mut tables_to_explore = 0;
        let mut potential_on_table = [0; ROUND_COUNT * TABLE_COUNT];
        for round in 0..ROUND_COUNT {
            for table in 0..TABLE_COUNT {
                tables_to_explore |= 1 << ((round << TO_EXPLORE_SHIFT) + table);
                potential_on_table[round * TABLE_COUNT + table] = (1 << 24) - 1;
            }
        }
        let mut state = Self {
            tables_to_explore,
            players_played_count: 0,
            empty_table_count: (ROUND_COUNT * TABLE_COUNT) as u8,
            players_played_with: [0; PLAYER_COUNT],
            played_in_round: [0; ROUND_COUNT],
            played_on_table: [0; ROUND_COUNT * TABLE_COUNT],
            potential_on_table,
            played_on_table_total: [0; TABLE_COUNT],
        };
        let mut player = 0;
        for table in 0..TABLE_COUNT {
            for _ in 0..PLAYERS_PER_TABLE {
                state.apply_player(0, table, player);
                player += 1;
            }
        }
        state
    }

    fn can_place_player_on_table<'b>(&mut self, round: usize, table: usize, player: usize) -> bool {
        self.players_played_with[player] & self.played_on_table[round * TABLE_COUNT + table] == 0
    }

    fn apply_player(&mut self, round: usize, table: usize, player: usize) {
        if round >= ROUND_COUNT || table >= TABLE_COUNT || player >= PLAYER_COUNT {
            unreachable!();
        }
        self.players_played_count += 1;
        let player_mask: u32 = 1 << player;
        let remove_player_mask: u32 = !player_mask;
        for r2 in 0..ROUND_COUNT {
            // Remove player from the table in other rounds
            self.potential_on_table[r2 * TABLE_COUNT + table] &= remove_player_mask;
        }
        for t2 in 0..TABLE_COUNT {
            // Remove player from other tables in the same round
            self.potential_on_table[round * TABLE_COUNT + t2] &= remove_player_mask;
        }

        // Add player to played in round
        self.played_in_round[round] |= player_mask;
        // Add player to played on table
        self.played_on_table_total[table] |= player_mask;

        let mut other_players = self.played_on_table[round * TABLE_COUNT + table];
        // Remove players current player has previously played with from tables potential
        self.potential_on_table[round * TABLE_COUNT + table] &= !self.players_played_with[player];
        // Add other players on table to current players played with list
        self.players_played_with[player] |= other_players;
        while other_players != 0 {
            let trailing_zeros = other_players.trailing_zeros() as usize;
            other_players &= !(1 << trailing_zeros);
            // Add current player to each other players played with list
            self.players_played_with[trailing_zeros] |= player_mask;
        }

        self.potential_on_table[round * TABLE_COUNT + table] |= player_mask;
        self.played_on_table[round * TABLE_COUNT + table] |= player_mask;
    }

    pub fn get_players_played_count(&self) -> u8 {
        self.players_played_count
    }

    pub fn step(&mut self) -> Result<Option<Self>, ()> {
        //self.find_hidden_singles(&mut buffer_1);

        let mut lowest: Option<(u32, usize, usize)> = None;
        let mut to_explore = self.tables_to_explore;
        while to_explore != 0 {
            let trailing_zeros = to_explore.trailing_zeros();
            to_explore &= !(1 << trailing_zeros);
            let round = trailing_zeros >> TO_EXPLORE_SHIFT;
            let table = trailing_zeros - (round << TO_EXPLORE_SHIFT);
            let round = round as usize;
            let table = table as usize;
            if table >= TABLE_COUNT || round >= ROUND_COUNT {
                self.tables_to_explore &= !(1 << trailing_zeros);
                continue;
            }

            let fixed_player_count = self.played_on_table[round * TABLE_COUNT + table].count_ones();
            match fixed_player_count.cmp(&(PLAYERS_PER_TABLE as u32)) {
                core::cmp::Ordering::Less => {
                    if self.potential_on_table[round * TABLE_COUNT + table].count_ones() as usize
                        == PLAYERS_PER_TABLE
                    {
                        loop {
                            let potential = self.potential_on_table[round * TABLE_COUNT + table]
                                & !self.played_on_table[round * TABLE_COUNT + table];
                            if potential != 0 {
                                let player = potential.trailing_zeros() as usize;
                                if self.can_place_player_on_table(round, table, player) {
                                    self.apply_player(round, table, player);
                                } else {
                                    self.potential_on_table[round * TABLE_COUNT + table] &=
                                        !(1 << player);
                                }
                            } else {
                                break;
                            };
                        }
                    } else {
                        lowest = Some(if let Some(lowest) = lowest {
                            if fixed_player_count < lowest.0 {
                                (fixed_player_count, round, table)
                            } else {
                                lowest
                            }
                        } else {
                            (fixed_player_count, round, table)
                        });
                    }
                }
                core::cmp::Ordering::Equal => {
                    self.tables_to_explore &= !(1 << trailing_zeros);
                    self.empty_table_count = self.empty_table_count.checked_sub(1).unwrap();
                    self.potential_on_table[round * TABLE_COUNT + table] =
                        self.played_on_table[round * TABLE_COUNT + table];
                    continue;
                }
                core::cmp::Ordering::Greater => return Err(()),
            }
        }
        if let Some((_, round, table)) = lowest {
            let potential = self.potential_on_table[round * TABLE_COUNT + table]
                & !self.played_on_table[round * TABLE_COUNT + table];
            let mut temp = potential;
            'played_iter: while temp != 0 {
                let player = temp.trailing_zeros() as usize;
                let player_bit = 1 << player;
                temp &= !player_bit;
                if !self.can_place_player_on_table(round, table, player) {
                    self.potential_on_table[round * TABLE_COUNT + table] &= !player_bit;
                    continue 'played_iter;
                }

                let mut state2 = *self;
                self.potential_on_table[round * TABLE_COUNT + table] &= !player_bit;
                state2.apply_player(round, table, player);
                return Ok(Some(state2));
            }
            return Err(());
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct Scheduler<'a> {
    tables: &'a [usize],
    round_range: RoundRange,
    table_range: TableRange,
    rounds: usize,
    player_count: usize,
    player_bit_word_count: usize,
    offsets: Offsets,
}

type ST = u64;

impl<'a> Scheduler<'a> {
    pub const fn new(tables: &'a [usize], rounds: usize) -> Self {
        let mut player_count: usize = 0;
        let mut i = 0;
        while i < tables.len() {
            player_count += tables[i];
            i += 1;
        }
        let player_bit_word_count =
            player_count / Self::word_size() + (player_count % Self::word_size() != 0) as usize;
        let played_with_size = player_bit_word_count * player_count;
        let played_on_table_total_size = player_bit_word_count * tables.len();
        let played_in_round_size = player_bit_word_count * rounds;
        let played_on_table_size = player_bit_word_count * tables.len() * rounds;
        let divisor = ToExplore::<ST>::calc_divisor(tables.len()) as usize;
        let mut to_explore_size = rounds * divisor * 2;
        to_explore_size = to_explore_size / Self::word_size()
            + (to_explore_size % Self::word_size() != 0) as usize;
        let offsets = Offsets::new(
            to_explore_size,
            divisor,
            played_with_size,
            played_on_table_total_size,
            played_in_round_size,
            played_on_table_size,
        );

        Self {
            tables,
            player_count,
            round_range: RoundRange::new(0, rounds),
            table_range: TableRange::new(0, tables.len()),
            rounds,
            player_bit_word_count,
            offsets,
        }
    }

    pub fn import_buffer<'b>(&self, buffer: &'b mut [u64]) -> Option<Schedule<'b>> {
        Schedule::import_buffer(buffer, self)
    }

    pub fn format_schedule<'b, W: core::fmt::Write>(
        &self,
        buffer: Schedule<'b>,
        output: &mut W,
    ) -> core::fmt::Result {
        fn base_10_length(n: usize) -> usize {
            (1..)
                .try_fold(n, |acc, i| if acc >= 10 { Ok(acc / 10) } else { Err(i) })
                .err()
                .unwrap_or(0)
        }
        output.write_str("     ")?;
        for table in 0..self.tables.len() {
            let now = table + 1;
            output.write_char('|')?;
            for _ in 0..(3 - base_10_length(now)) {
                output.write_char(' ')?;
            }
            output.write_fmt(format_args!("{}", now))?;
            output.write_str("  ")?;
        }

        for round in 0..self.tables.len() {
            output.write_str("\n-----")?;
            for _ in 0..self.tables.len() {
                output.write_char('+')?;
                output.write_str("-----")?;
            }
            for i in 0..(self.player_count / self.tables.len() + 1) {
                if i == (self.player_count / self.tables.len() + 1) / 2 {
                    output.write_char('\n')?;
                    let now = round + 1;
                    for _ in 0..(3 - base_10_length(now)) {
                        output.write_char(' ')?;
                    }
                    output.write_fmt(format_args!("{}", now))?;
                    output.write_str("  ")?;
                } else {
                    output.write_str("\n     ")?;
                }
                'table: for table in 0..self.tables.len() {
                    output.write_char('|')?;
                    let mut counter = 0;
                    for byte in 0..self.player_bit_word_count {
                        let mut temp = buffer.rest[self.offsets.played_on_table_offset
                            + self.player_bit_word_count * (round * self.tables.len() + table)
                            + byte];
                        while temp != 0 {
                            let trailing_zeros = temp.trailing_zeros() as usize;
                            let player = byte * Self::word_size() + trailing_zeros;
                            let player_bit = 1 << trailing_zeros;
                            temp &= !player_bit;
                            if counter == i {
                                let now = player;
                                for _ in 0..(3 - base_10_length(now)) {
                                    output.write_char(' ')?;
                                }
                                output.write_fmt(format_args!("{}", now))?;
                                output.write_str("  ")?;
                                continue 'table;
                            }
                            counter += 1;
                        }
                    }

                    output.write_str("     ")?;
                }
            }
        }
        Ok(())
    }

    pub fn get_schedule<'b>(&self, buffer: Schedule<'b>) -> &'b [u64] {
        &buffer.rest[self.offsets.played_on_table_offset..][..self.offsets.played_on_table_size]
    }

    pub const fn get_block_size(&self) -> usize {
        self.offsets.block_size
    }

    #[must_use]
    pub fn initialise_buffer(&self, buffer: &mut [u64]) -> bool {
        buffer.fill(0);
        let mut buffer = if let Some(buffer) = self.import_buffer(buffer) {
            buffer
        } else {
            return false;
        };

        let max = Self::get_byte_and_mask(self.player_count);
        let start =
            self.offsets.potential_on_table_offset + self.player_bit_word_count * self.tables.len(); // Skip first round
        let end = self.offsets.potential_on_table_offset + self.offsets.played_on_table_size;
        let mut i = 0;
        while start + i < end {
            let current_byte = i % self.player_bit_word_count;
            buffer.rest[start + i] = if current_byte > 0 {
                0
            } else if current_byte == max.0 {
                max.1 - 1
            } else {
                u64::MAX
            };
            i += 1;
        }

        buffer.rest[self.offsets.empty_table_count_offset] =
            ((self.rounds - 1) * self.tables.len()) as u64;
        let mut round_range = self.round_range.skip(1);
        while let Some(round) = round_range.next() {
            let mut table_range = self.table_range;
            while let Some(table) = table_range.next() {
                buffer
                    .to_explore
                    .set(round.as_usize(), table.as_usize(), true);
            }
        }

        let mut pos = 0;
        let mut table_range = self.table_range;
        while let Some(table_number) = table_range.next() {
            let size = self.tables[table_number.as_usize()];
            let mut player = pos;
            let zero = if let Some(zero) = self.round_range.convert_usize(0) {
                zero
            } else {
                return false;
            };
            while player < pos + size {
                self.apply_player(&mut buffer, zero, table_number, player);
                player += 1;
            }
            pos += size;
        }
        true
    }

    const fn word_size() -> usize {
        core::mem::size_of::<u64>() * 8
    }
    const fn get_byte_and_mask(player: usize) -> (usize, u64) {
        let byte = player / Self::word_size();
        let mask = 1 << (player - (byte * Self::word_size()));
        (byte, mask)
    }

    fn apply_player<'b>(
        &self,
        buffer: &mut Schedule<'b>,
        round: Round,
        table: Table,
        player: usize,
    ) -> Option<()> {
        if round.as_usize() >= self.rounds
            || table.as_usize() >= self.tables.len()
            || player >= self.player_count
        {
            return None;
        }
        let (byte, player_mask) = Self::get_byte_and_mask(player);
        let remove_player_mask = !player_mask;
        buffer.rest[self.offsets.players_placed_counter_offset] += 1; // Will double count if called multiple times
        {
            let mut r2 = 0;
            while r2 < self.rounds {
                // Remove player from the table in other rounds
                buffer.rest[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count * (r2 * self.tables.len() + table.as_usize())
                    + byte] &= remove_player_mask;
                r2 += 1;
            }
        }
        {
            let mut t2 = 0;
            while t2 < self.tables.len() {
                // Remove player from other tables in the same round
                buffer.rest[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count * (round.as_usize() * self.tables.len() + t2)
                    + byte] &= remove_player_mask;
                t2 += 1;
            }
        }
        // Add player to played in round
        buffer.rest[self.offsets.played_in_round_offset
            + self.player_bit_word_count * round.as_usize()
            + byte] |= player_mask;
        // Add player to played on table
        buffer.rest[self.offsets.played_on_table_total_offset
            + self.player_bit_word_count * table.as_usize()
            + byte] |= player_mask;

        {
            let mut other_byte = 0;
            while other_byte < self.player_bit_word_count {
                let mut other_players = buffer.rest[self.offsets.played_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + other_byte];

                buffer.rest[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + other_byte] &= !buffer.rest[self.offsets.played_with_offset
                    + self.player_bit_word_count * player
                    + other_byte];

                // Add other players to players played with
                buffer.rest[self.offsets.played_with_offset
                    + self.player_bit_word_count * player
                    + other_byte] |= other_players;
                while other_players != 0 {
                    let trailing_zeros = other_players.trailing_zeros() as usize;
                    let other_player = other_byte * Self::word_size() + trailing_zeros;
                    let other_player_bit = 1 << trailing_zeros;
                    other_players &= !other_player_bit;

                    // Add player to other players played with
                    buffer.rest[self.offsets.played_with_offset
                        + self.player_bit_word_count * other_player
                        + byte] |= player_mask;
                }

                other_byte += 1;
            }
        }

        // Add player to their own table+round
        buffer.rest[self.offsets.potential_on_table_offset
            + self.player_bit_word_count
                * (round.as_usize() * self.tables.len() + table.as_usize())
            + byte] |= player_mask;
        buffer.rest[self.offsets.played_on_table_offset
            + self.player_bit_word_count
                * (round.as_usize() * self.tables.len() + table.as_usize())
            + byte] |= player_mask;
        Some(())
    }

    pub const fn get_players_placed<'b>(&self, buffer: Schedule<'b>) -> u64 {
        buffer.rest[self.offsets.players_placed_counter_offset]
    }

    pub const fn get_empty_table_count<'b>(&self, buffer: Schedule<'b>) -> u64 {
        buffer.rest[self.offsets.empty_table_count_offset]
    }

    pub fn find_hidden_singles<'b>(&self, buffer: &mut Schedule<'b>) {
        let mut round_range = self.round_range;
        while let Some(round) = round_range.next() {
            let mut byte = 0;
            while byte < self.player_bit_word_count {
                let mut potential_in_row = !buffer.rest[self.offsets.played_in_round_offset
                    + self.player_bit_word_count * round.as_usize()
                    + byte];
                'loop_bits_round: while potential_in_row != 0 {
                    let trailing_zeros = potential_in_row.trailing_zeros() as usize;
                    let player = byte * Self::word_size() + trailing_zeros;
                    let player_bit: u64 = 1 << trailing_zeros;
                    potential_in_row &= !player_bit;
                    if player >= self.player_count {
                        break;
                    }
                    let mut only_position = None;
                    let mut table_range = self.table_range;
                    while let Some(table) = table_range.next() {
                        if buffer.rest[self.offsets.potential_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte]
                            & player_bit
                            != 0
                        {
                            if only_position.is_none() {
                                only_position = Some(table);
                            } else {
                                continue 'loop_bits_round;
                            }
                        }
                    }
                    if let Some(table) = only_position {
                        //println!("Found single location: {:?}", (round, table, player));
                        self.apply_player(buffer, round, table, player);
                    }
                }
                byte += 1;
            }
        }

        let mut table_range = self.table_range;
        while let Some(table) = table_range.next() {
            let mut byte = 0;
            while byte < self.player_bit_word_count {
                let mut potential_in_column =
                    !buffer.rest[self.offsets.played_on_table_total_offset
                        + self.player_bit_word_count * table.as_usize()
                        + byte];
                'loop_bits_table: while potential_in_column != 0 {
                    let trailing_zeros = potential_in_column.trailing_zeros() as usize;
                    let player = byte * Self::word_size() + trailing_zeros;
                    let player_bit = 1 << trailing_zeros;
                    potential_in_column &= !player_bit;
                    if player >= self.player_count {
                        break;
                    }
                    let mut only_position = None;
                    let mut round_range = self.round_range;
                    while let Some(round) = round_range.next() {
                        if buffer.rest[self.offsets.potential_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte]
                            & player_bit
                            != 0
                        {
                            if only_position.is_none() {
                                only_position = Some(round);
                            } else {
                                continue 'loop_bits_table;
                            }
                        }
                    }
                    if let Some(round) = only_position {
                        //println!("Found single location: {:?}", (round, table, player));
                        self.apply_player(buffer, round, table, player);
                    }
                }
                byte += 1;
            }
        }
    }

    const fn get_fixed_count<'b>(&self, buffer: &Schedule<'b>, round: Round, table: Table) -> u32 {
        let mut fixed_player_count = 0;
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            fixed_player_count += buffer.rest[self.offsets.played_on_table_offset
                + self.player_bit_word_count
                    * (round.as_usize() * self.tables.len() + table.as_usize())
                + byte]
                .count_ones();
            byte += 1;
        }
        fixed_player_count
    }

    const fn get_potential_count<'b>(
        &self,
        buffer: &Schedule<'b>,
        round: Round,
        table: Table,
    ) -> u32 {
        let mut potential_player_count = 0;
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            potential_player_count += buffer.rest[self.offsets.potential_on_table_offset
                + self.player_bit_word_count
                    * (round.as_usize() * self.tables.len() + table.as_usize())
                + byte]
                .count_ones();
            byte += 1;
        }
        potential_player_count
    }

    const fn can_place_player_on_table<'b>(
        &self,
        buffer: &Schedule<'b>,
        round: Round,
        table: Table,
        player: usize,
    ) -> bool {
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            if buffer.rest
                [self.offsets.played_with_offset + self.player_bit_word_count * player + byte]
                & buffer.rest[self.offsets.played_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + byte]
                != 0
            {
                return false;
            }
            byte += 1;
        }
        true
    }

    pub fn step(&self, buffer_1: &mut [u64], buffer_2: &mut [u64]) -> Option<bool> {
        let buffer_1 = &mut buffer_1[..self.offsets.block_size];
        let buffer_2 = &mut buffer_2[..self.offsets.block_size];

        let mut buffer_1 = self.import_buffer(buffer_1)?;
        let mut buffer_2 = self.import_buffer(buffer_2)?;

        self.find_hidden_singles(&mut buffer_1);
        let offset = self.offsets.potential_on_table_offset;

        let mut lowest: Option<(u32, Round, Table)> = None;
        let (mut buffer_1, mut to_explore) = buffer_1.extract_to_explore();
        let (mut buffer_2, mut to_explore_2) = buffer_2.extract_to_explore();
        let mut to_explore = to_explore.iter_mut();
        while let Some((round, table)) = to_explore.next() {
            let (round, table) = if let Some(val) = self
                .round_range
                .convert_usize(round)
                .zip(self.table_range.convert_usize(table))
            {
                val
            } else {
                to_explore.remove(round, table);
                // If round or table is out of bounds, then remove
                continue;
            };
            let table_size = self.tables[table.as_usize()] as u32;

            let fixed_player_count = self.get_fixed_count(&mut buffer_1, round, table);

            match fixed_player_count.cmp(&table_size) {
                core::cmp::Ordering::Less => {
                    if self.get_potential_count(&mut buffer_1, round, table) == table_size {
                        let potential_index = self.offsets.potential_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize());
                        let fixed_index = self.offsets.played_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize());
                        for byte in 0..self.player_bit_word_count {
                            loop {
                                let potential = buffer_1.rest[potential_index + byte]
                                    & !buffer_1.rest[fixed_index + byte];
                                if potential != 0 {
                                    let trailing_zeros = potential.trailing_zeros() as usize;
                                    let player = byte * Self::word_size() + trailing_zeros;
                                    let player_bit = 1 << trailing_zeros;
                                    if self.can_place_player_on_table(
                                        &mut buffer_1,
                                        round,
                                        table,
                                        player,
                                    ) {
                                        self.apply_player(&mut buffer_1, round, table, player);
                                    } else {
                                        buffer_1.rest[potential_index + byte] &= !player_bit;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    } else {
                        lowest = Some(if let Some(lowest) = lowest {
                            if fixed_player_count < lowest.0 {
                                (fixed_player_count, round, table)
                            } else {
                                lowest
                            }
                        } else {
                            (fixed_player_count, round, table)
                        });
                    }
                }
                core::cmp::Ordering::Equal => {
                    to_explore.remove(round.as_usize(), table.as_usize());

                    buffer_1.rest[self.offsets.empty_table_count_offset] -= 1;
                    for byte in 0..self.player_bit_word_count {
                        // Set potential to fixed players
                        buffer_1.rest[self.offsets.potential_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte] = buffer_1.rest[self.offsets.played_on_table_offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte]
                    }
                    continue;
                }
                core::cmp::Ordering::Greater => return None,
            }
        }

        if let Some((_, round, table)) = lowest {
            for byte in 0..self.player_bit_word_count {
                let fixed = buffer_1.rest[self.offsets.played_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + byte];
                let potential = buffer_1.rest[offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + byte]
                    & !fixed;
                let mut temp = potential;
                'played_iter: while temp != 0 {
                    let trailing_zeros = temp.trailing_zeros() as usize;
                    let player = byte * Self::word_size() + trailing_zeros;
                    let player_bit = 1 << trailing_zeros;
                    temp &= !player_bit;
                    if !self.can_place_player_on_table(&mut buffer_1, round, table, player) {
                        // If player has already played with any of the players then remove the player from the potential
                        buffer_1.rest[offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte] &= !player_bit;
                        continue 'played_iter;
                    }

                    //buffer_2.copy_from_slice(buffer_1);
                    for i in 0..self.offsets.block_size - self.offsets.to_explore_size {
                        buffer_2.rest[i] = buffer_1.rest[i];
                    }
                    to_explore_2
                        .get_data()
                        .copy_from_slice(to_explore.get_data());
                    buffer_1.rest[offset
                        + self.player_bit_word_count
                            * (round.as_usize() * self.tables.len() + table.as_usize())
                        + byte] &= !player_bit;
                    self.apply_player(&mut buffer_2, round, table, player);
                    return Some(false);
                }
            }
            return None; // Could not place any player but fixed_player_count < table_size
        }
        Some(true)
    }
}
