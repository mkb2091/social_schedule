use crate::util::*;
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

pub trait Schedule<T: Word> {
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
    a: Box<dyn Schedule<usize>>,
}

#[derive(Debug)]
pub enum SchedulerResult {}

#[derive(Debug)]
struct Offsets {
    players_placed_counter_offset: usize,
    empty_table_count_offset: usize,
    to_explore_offset: usize,
    to_explore_size: usize,
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
        played_with_size: usize,
        played_on_table_total_size: usize,
        played_in_round_size: usize,
        played_on_table_size: usize,
    ) -> Self {
        let players_placed_counter_offset = 0;
        let empty_table_count_offset = players_placed_counter_offset + 1;
        let to_explore_offset = empty_table_count_offset + 1;
        let played_with_offset = to_explore_offset + to_explore_size;
        let played_on_table_total_offset = played_with_offset + played_with_size;
        let played_in_round_offset = played_on_table_total_offset + played_on_table_total_size;
        let played_on_table_offset = played_in_round_offset + played_in_round_size;
        let potential_on_table_offset = played_on_table_offset + played_on_table_size;
        let block_size = potential_on_table_offset + played_on_table_size;
        Self {
            players_placed_counter_offset,
            empty_table_count_offset,
            to_explore_offset,
            to_explore_size,
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
        let mut to_explore_size = rounds * tables.len() * 2;
        to_explore_size = to_explore_size / Self::word_size()
            + (to_explore_size % Self::word_size() != 0) as usize;
        let offsets = Offsets::new(
            to_explore_size,
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

    pub fn format_schedule<W: core::fmt::Write>(
        &self,
        buffer: &[usize],
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
                        let mut temp = buffer[self.offsets.played_on_table_offset
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

    pub fn get_schedule<'b>(&self, buffer: &'b [u64]) -> &'b [u64] {
        &buffer[self.offsets.played_on_table_offset..][..self.offsets.played_on_table_size]
    }

    pub const fn get_block_size(&self) -> usize {
        self.offsets.block_size
    }

    #[must_use]
    pub fn initialise_buffer(&self, buffer: &mut [u64]) -> bool {
        if buffer.len() < self.offsets.block_size {
            return false;
        }
        let mut i = 0;
        while i < self.offsets.block_size {
            buffer[i] = 0;
            i += 1;
        }

        let max = Self::get_byte_and_mask(self.player_count);
        let start =
            self.offsets.potential_on_table_offset + self.player_bit_word_count * self.tables.len(); // Skip first round
        let end = self.offsets.potential_on_table_offset + self.offsets.played_on_table_size;
        let mut i = 0;
        while start + i < end {
            let current_byte = i % self.player_bit_word_count;
            buffer[start + i] = if current_byte > 0 {
                0
            } else if current_byte == max.0 {
                max.1 - 1
            } else {
                u64::MAX
            };
            i += 1;
        }

        buffer[self.offsets.empty_table_count_offset] =
            ((self.rounds - 1) * self.tables.len()) as u64;
        let mut round_range = self.round_range.skip(1);
        while let Some(round) = round_range.next() {
            let mut table_range = self.table_range;
            while let Some(table) = table_range.next() {
                let number = round.as_usize() * self.tables.len() + table.as_usize();
                let byte = number / Self::word_size();
                let mask = 1 << (number - (byte * Self::word_size()));
                buffer[self.offsets.to_explore_offset] |= mask;
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
                self.apply_player(buffer, zero, table_number, player);
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

    fn apply_player(
        &self,
        buffer: &mut [u64],
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
        buffer[self.offsets.players_placed_counter_offset] += 1; // Will double count if called multiple times
        {
            let mut r2 = 0;
            while r2 < self.rounds {
                // Remove player from the table in other rounds
                buffer[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count * (r2 * self.tables.len() + table.as_usize())
                    + byte] &= remove_player_mask;
                r2 += 1;
            }
        }
        {
            let mut t2 = 0;
            while t2 < self.tables.len() {
                // Remove player from other tables in the same round
                buffer[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count * (round.as_usize() * self.tables.len() + t2)
                    + byte] &= remove_player_mask;
                t2 += 1;
            }
        }
        // Add player to played in round
        buffer[self.offsets.played_in_round_offset
            + self.player_bit_word_count * round.as_usize()
            + byte] |= player_mask;
        // Add player to played on table
        buffer[self.offsets.played_on_table_total_offset
            + self.player_bit_word_count * table.as_usize()
            + byte] |= player_mask;

        {
            let mut other_byte = 0;
            while other_byte < self.player_bit_word_count {
                let mut other_players = buffer[self.offsets.played_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + other_byte];

                buffer[self.offsets.potential_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + other_byte] &= !buffer[self.offsets.played_with_offset
                    + self.player_bit_word_count * player
                    + other_byte];

                // Add other players to players played with
                buffer[self.offsets.played_with_offset
                    + self.player_bit_word_count * player
                    + other_byte] |= other_players;
                while other_players != 0 {
                    let trailing_zeros = other_players.trailing_zeros() as usize;
                    let other_player = other_byte * Self::word_size() + trailing_zeros;
                    let other_player_bit = 1 << trailing_zeros;
                    other_players &= !other_player_bit;

                    // Add player to other players played with
                    buffer[self.offsets.played_with_offset
                        + self.player_bit_word_count * other_player
                        + byte] |= player_mask;
                }

                other_byte += 1;
            }
        }

        // Add player to their own table+round
        buffer[self.offsets.potential_on_table_offset
            + self.player_bit_word_count
                * (round.as_usize() * self.tables.len() + table.as_usize())
            + byte] |= player_mask;
        buffer[self.offsets.played_on_table_offset
            + self.player_bit_word_count
                * (round.as_usize() * self.tables.len() + table.as_usize())
            + byte] |= player_mask;
        Some(())
    }

    pub const fn get_players_placed(&self, buffer: &[u64]) -> u64 {
        buffer[self.offsets.players_placed_counter_offset]
    }

    pub const fn get_empty_table_count(&self, buffer: &[u64]) -> u64 {
        buffer[self.offsets.empty_table_count_offset]
    }

    pub fn find_hidden_singles(&self, buffer: &mut [u64]) {
        let mut round_range = self.round_range;
        while let Some(round) = round_range.next() {
            let mut byte = 0;
            while byte < self.player_bit_word_count {
                let mut potential_in_row = !buffer[self.offsets.played_in_round_offset
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
                        if buffer[self.offsets.potential_on_table_offset
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
                let mut potential_in_column = !buffer[self.offsets.played_on_table_total_offset
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
                        if buffer[self.offsets.potential_on_table_offset
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

    const fn get_fixed_count(&self, buffer: &[u64], round: Round, table: Table) -> u32 {
        let mut fixed_player_count = 0;
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            fixed_player_count += buffer[self.offsets.played_on_table_offset
                + self.player_bit_word_count
                    * (round.as_usize() * self.tables.len() + table.as_usize())
                + byte]
                .count_ones();
            byte += 1;
        }
        fixed_player_count
    }

    const fn get_potential_count(&self, buffer: &[u64], round: Round, table: Table) -> u32 {
        let mut potential_player_count = 0;
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            potential_player_count += buffer[self.offsets.potential_on_table_offset
                + self.player_bit_word_count
                    * (round.as_usize() * self.tables.len() + table.as_usize())
                + byte]
                .count_ones();
            byte += 1;
        }
        potential_player_count
    }

    const fn can_place_player_on_table(
        &self,
        buffer: &[u64],
        round: Round,
        table: Table,
        player: usize,
    ) -> bool {
        let mut byte = 0;
        while byte < self.player_bit_word_count {
            if buffer[self.offsets.played_with_offset + self.player_bit_word_count * player + byte]
                & buffer[self.offsets.played_on_table_offset
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

        self.find_hidden_singles(buffer_1);
        let offset = self.offsets.potential_on_table_offset;

        let mut lowest: Option<(u32, Round, Table)> = None;
        for to_explore_byte in 0..self.offsets.to_explore_size {
            let mut to_explore = buffer_1[self.offsets.to_explore_offset + to_explore_byte];
            while to_explore != 0 {
                let trailing_zeros = to_explore.trailing_zeros();
                let number = to_explore_byte * Self::word_size() + trailing_zeros as usize;
                to_explore &= !(1 << trailing_zeros);
                let (round, table) = if let Some(val) = self
                    .round_range
                    .convert_usize(number / self.tables.len())
                    .zip(self.table_range.convert_usize(number % self.tables.len()))
                {
                    val
                } else {
                    buffer_1[self.offsets.to_explore_offset + to_explore_byte] &=
                        !(1 << trailing_zeros);
                    // If round or table is out of bounds, then remove
                    continue;
                };
                let table_size = self.tables[table.as_usize()] as u32;

                let fixed_player_count = self.get_fixed_count(buffer_1, round, table);

                match fixed_player_count.cmp(&table_size) {
                    core::cmp::Ordering::Less => {
                        if self.get_potential_count(buffer_1, round, table) == table_size {
                            let potential_index = self.offsets.potential_on_table_offset
                                + self.player_bit_word_count
                                    * (round.as_usize() * self.tables.len() + table.as_usize());
                            let fixed_index = self.offsets.played_on_table_offset
                                + self.player_bit_word_count
                                    * (round.as_usize() * self.tables.len() + table.as_usize());
                            for byte in 0..self.player_bit_word_count {
                                loop {
                                    let potential = buffer_1[potential_index + byte]
                                        & !buffer_1[fixed_index + byte];
                                    if potential != 0 {
                                        let trailing_zeros = potential.trailing_zeros() as usize;
                                        let player = byte * Self::word_size() + trailing_zeros;
                                        let player_bit = 1 << trailing_zeros;
                                        if self.can_place_player_on_table(
                                            buffer_1, round, table, player,
                                        ) {
                                            self.apply_player(buffer_1, round, table, player);
                                        } else {
                                            buffer_1[potential_index + byte] &= !player_bit;
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
                        buffer_1[self.offsets.to_explore_offset + to_explore_byte] &=
                            !(1 << trailing_zeros);

                        buffer_1[self.offsets.empty_table_count_offset] -= 1;
                        for byte in 0..self.player_bit_word_count {
                            // Set potential to fixed players
                            buffer_1[self.offsets.potential_on_table_offset
                                + self.player_bit_word_count
                                    * (round.as_usize() * self.tables.len() + table.as_usize())
                                + byte] = buffer_1[self.offsets.played_on_table_offset
                                + self.player_bit_word_count
                                    * (round.as_usize() * self.tables.len() + table.as_usize())
                                + byte]
                        }
                        continue;
                    }
                    core::cmp::Ordering::Greater => return None,
                }
            }
        }

        if let Some((_, round, table)) = lowest {
            for byte in 0..self.player_bit_word_count {
                let fixed = buffer_1[self.offsets.played_on_table_offset
                    + self.player_bit_word_count
                        * (round.as_usize() * self.tables.len() + table.as_usize())
                    + byte];
                let potential = buffer_1[offset
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
                    if !self.can_place_player_on_table(buffer_1, round, table, player) {
                        // If player has already played with any of the players then remove the player from the potential
                        buffer_1[offset
                            + self.player_bit_word_count
                                * (round.as_usize() * self.tables.len() + table.as_usize())
                            + byte] &= !player_bit;
                        continue 'played_iter;
                    }

                    //buffer_2.copy_from_slice(buffer_1);
                    for i in 0..self.offsets.block_size {
                        buffer_2[i] = buffer_1[i];
                    }
                    buffer_1[offset
                        + self.player_bit_word_count
                            * (round.as_usize() * self.tables.len() + table.as_usize())
                        + byte] &= !player_bit;
                    self.apply_player(buffer_2, round, table, player);
                    return Some(false);
                }
            }
            return None; // Could not place any player but fixed_player_count < table_size
        }
        Some(true)
    }
}
