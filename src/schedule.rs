use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::ops::IndexMut;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Schedule {
    player_count: usize,
    tables: usize,
    matches: Vec<u64>,
    player_positions: Vec<usize>,
    player_opponent_cache: Vec<u32>,
    unique_games_played_cache: u32,
    unique_opponent_sum_cache: u32,
    score_multiplier: u32,
}

impl Schedule {
    pub fn new(player_count: usize, tables: usize) -> Schedule {
        let mut matches: Vec<u64> = Vec::with_capacity(tables * tables);
        for _ in 0..(tables * tables) {
            matches.push(0);
        }
        let mut player_positions: Vec<usize> = Vec::with_capacity(player_count * tables);
        for _ in 0..(player_count * tables) {
            player_positions.push(0);
        }
        let mut player_opponent_cache: Vec<u32> = Vec::with_capacity(player_count);
        for _ in 0..player_count {
            player_opponent_cache.push(0);
        }
        Schedule {
            player_count,
            tables,
            matches,
            player_positions,
            player_opponent_cache,
            unique_games_played_cache: 0,
            unique_opponent_sum_cache: 0,
            score_multiplier: 2 * (player_count - tables) as u32,
        }
    }

    pub fn from_vec(player_count: usize, tables: usize, data: Vec<Vec<Vec<usize>>>) -> Schedule {
        let mut new = Schedule::new(player_count, tables);
        new.import_vec(data);
        new
    }

    pub fn import_vec(&mut self, data: Vec<Vec<Vec<usize>>>) {
        self.matches = Vec::with_capacity(self.tables * self.tables);
        for _ in 0..(self.tables * self.tables) {
            self.matches.push(0);
        }
        self.player_positions = Vec::with_capacity(self.player_count * self.tables);
        for _ in 0..(self.player_count * self.tables) {
            self.player_positions.push(0);
        }
        self.player_opponent_cache = Vec::with_capacity(self.player_count);
        for _ in 0..self.player_count {
            self.player_opponent_cache.push(0);
        }
        for (round_number, round) in data.iter().enumerate() {
            for (table_number, table) in round.iter().enumerate() {
                for player in table.iter() {
                    *self.get_mut(round_number, table_number) |= 1_u64 << player;
                    self.player_positions[player * self.tables + round_number] =
                        round_number * self.tables + table_number;
                }
            }
        }
        self.find_unique_opponents();
        self.find_unique_games_played();
    }

    pub fn generate_random<T: rand::Rng + rand_core::RngCore>(&mut self, rng: &mut T) {
        let players: Vec<usize> = (0..self.player_count).collect();
        let mut game: Vec<Vec<Vec<usize>>> = Vec::new();
        for _round in 0..self.tables {
            let mut player_list = players.clone();
            player_list.shuffle(rng);
            let mut round: Vec<Vec<usize>> = Vec::new();
            for _ in 0..self.tables {
                round.push(Vec::new());
            }
            for (i, player) in player_list.iter().enumerate() {
                round[i % self.tables].push(*player);
            }
            game.push(round);
        }
        self.import_vec(game);
    }

    fn get(&self, round: usize, table: usize) -> u64 {
        self.matches[round * self.tables + table]
    }

    fn get_mut(&mut self, round: usize, table: usize) -> &mut u64 {
        self.matches.index_mut(round * self.tables + table)
    }
    #[inline(never)]
    pub fn find_unique_games_played(&mut self) -> u32 {
        assert!(self.matches.len() >= self.tables * self.tables);
        let mut total: u32 = 0;
        for table1 in (0..(self.tables / 2 * 2)).step_by(2) {
            let table2 = table1 + 1;
            let mut total_table: u128 = 0;
            for round in 0..self.tables {
                let base = round * self.tables;
                total_table |= unsafe {
                    std::mem::transmute::<[u64; 2], u128>([
                        *self.matches.get_unchecked(base + table1),
                        *self.matches.get_unchecked(base + table2),
                    ])
                };
            }
            total += total_table.count_ones();
        }
        if self.tables % 2 == 1 {
            let mut total_table = 0;
            for round in 0..self.tables {
                total_table |= self.get(round, self.tables - 1);
            }
            total += total_table.count_ones();
        }
        debug_assert!(
            total == {
                let mut total2: u32 = 0;
                for table in 0..self.tables {
                    let mut total_table = 0;
                    for round in 0..self.tables {
                        total_table |= self.get(round, table);
                    }
                    total2 += total_table.count_ones();
                }
                total2
            }
        );
        self.unique_games_played_cache = total;
        total
    }
    pub fn unique_games_played(&self) -> u32 {
        self.unique_games_played_cache
    }
    fn player_unique_opponents(&mut self, player: usize) -> u32 {
        let count = (0..self.tables)
            .map(|round_number| {
                self.matches[self.player_positions[player * self.tables + round_number]]
            })
            .fold(0, |acc, round| acc | round)
            .count_ones();
        self.player_opponent_cache[player] = count;
        count
    }
    pub fn find_unique_opponents(&mut self) -> u32 {
        let mut total: u32 = 0;
        for player in 0..self.player_count {
            total += self.player_unique_opponents(player)
        }
        total -= self.player_count as u32;
        self.unique_opponent_sum_cache = total;
        total
    }

    fn sum_unique_opponent(&mut self) {
        self.unique_opponent_sum_cache =
            self.player_opponent_cache.iter().sum::<u32>() - self.player_count as u32;
    }
    pub fn unique_opponents(&self) -> u32 {
        self.unique_opponent_sum_cache
    }

    pub fn get_tables(&self) -> usize {
        self.tables
    }

    pub fn get_player_count(&self) -> usize {
        self.player_count
    }

    pub fn get_game(&self, round: usize, table: usize) -> Vec<usize> {
        let game = self.get(round, table);
        let mut players: Vec<usize> = Vec::with_capacity(game.count_ones() as usize);
        for player in 0..self.player_count {
            let player_number = 1 << player;
            if game & player_number != 0 {
                players.push(player);
            }
        }
        players
    }
    pub fn generate_score(&mut self) -> u32 {
        self.find_unique_opponents() * (self.tables as u32)
            + self.find_unique_games_played() * self.score_multiplier
    }
    pub fn get_score(&self) -> u32 {
        self.unique_opponents() * (self.tables as u32)
            + self.unique_games_played() * self.score_multiplier
    }
    pub fn improve_table(
        &mut self,
        old_score: u32,
        round: usize,
        table1: usize,
        table2: usize,
        apply: bool,
    ) -> (u32, u32) {
        let mut score = old_score;
        debug_assert!(score == self.get_score());
        debug_assert!(score == self.generate_score());
        let old_unique_games_played = self.unique_games_played();
        let mut unique_games_played = old_unique_games_played;
        let original_t1 = self.get(round, table1);
        let original_t2 = self.get(round, table2);
        let mut best_t1 = original_t1;
        let mut best_t2 = original_t2;
        let mut best_player1 = 0;
        let mut best_player2 = 0;
        let t1_size: usize = original_t1.count_ones() as usize;
        let t2_size: usize = original_t2.count_ones() as usize;
        let mut t1_players: Vec<usize> = Vec::with_capacity(t1_size);
        let mut t2_players: Vec<usize> = Vec::with_capacity(t2_size);
        for player1 in 0..self.player_count {
            if original_t1 & (1 << player1) != 0 {
                t1_players.push(player1);
                if t1_players.len() >= t1_size {
                    break;
                }
            }
        }
        for player2 in 0..self.player_count {
            if original_t2 & (1 << player2) != 0 {
                t2_players.push(player2);
                if t2_players.len() >= t2_size {
                    break;
                }
            }
        }
        let mut new_player_opponent_cache = self.player_opponent_cache.clone();
        for player1 in t1_players.iter() {
            let player_number1: u64 = 1_u64 << player1;
            for player2 in t2_players.iter() {
                let player_number2: u64 = 1_u64 << player2;
                *self.get_mut(round, table1) = original_t1 - player_number1 + player_number2;
                *self.get_mut(round, table2) = original_t2 - player_number2 + player_number1;
                self.player_positions
                    .swap(player1 * self.tables + round, player2 * self.tables + round);
                for p1 in t1_players.iter() {
                    self.player_unique_opponents(*p1);
                }
                for p2 in t2_players.iter() {
                    self.player_unique_opponents(*p2);
                }
                self.sum_unique_opponent();
                let new_unique_games_played = self.find_unique_games_played();
                let new_score = self.get_score();
                self.player_positions
                    .swap(player1 * self.tables + round, player2 * self.tables + round);
                if new_score > score
                    || (new_score == score && new_unique_games_played > unique_games_played)
                {
                    best_t1 = original_t1 - player_number1 + player_number2;
                    best_t2 = original_t2 - player_number2 + player_number1;
                    best_player1 = *player1;
                    best_player2 = *player2;
                    score = new_score;
                    unique_games_played = new_unique_games_played;
                    if apply {
                        new_player_opponent_cache = self.player_opponent_cache.clone();
                    }
                }
            }
        }
        self.player_opponent_cache = new_player_opponent_cache;
        if apply {
            *self.get_mut(round, table1) = best_t1;
            *self.get_mut(round, table2) = best_t2;
            if score > old_score {
                self.player_positions.swap(
                    best_player1 * self.tables + round,
                    best_player2 * self.tables + round,
                );
            }
            self.unique_games_played_cache = unique_games_played;
        } else {
            *self.get_mut(round, table1) = original_t1;
            *self.get_mut(round, table2) = original_t2;
            self.unique_games_played_cache = old_unique_games_played;
        }
        self.sum_unique_opponent();
        debug_assert!(self.get_score() == self.generate_score());
        (score, unique_games_played)
    }
}

pub struct Generator<T: rand::Rng + rand_core::RngCore> {
    player_count: usize,
    tables: usize,
    pub best: Schedule,
    pub best_score: u32,
    current: Schedule,
    current_score: u32,
    next: (usize, usize, usize),
    next_score: u32,
    next_unique_games: u32,
    rng: T,
    round: usize,
    table1: usize,
    table2: usize,
}

impl<T: rand::Rng + rand_core::RngCore> Generator<T> {
    pub fn new(mut rng: T, player_count: usize, tables: usize) -> Generator<T> {
        let mut best = Schedule::new(player_count, tables);
        best.generate_random(&mut rng);
        let score = best.generate_score();
        Generator {
            player_count,
            tables,
            best: best.clone(),
            best_score: score,
            current: best,
            current_score: score,
            next: (0, 0, 1),
            next_score: score,
            next_unique_games: 0,
            rng,
            round: 0,
            table1: 0,
            table2: 0,
        }
    }
    pub fn process(&mut self) {
        self.table2 += 1;
        if self.table2 >= self.tables {
            self.table2 = self.table1 + 2;
            self.table1 += 1;
            if self.table2 >= self.tables {
                self.table1 = 0;
                self.table2 = 1;
                self.round += 1;
                if self.round >= self.tables {
                    self.round = 0;
                    self.table1 = 0;
                    self.table2 = 1;
                    if self.next_score > self.current_score {
                        let (round, table1, table2) = self.next;
                        self.current
                            .improve_table(self.current_score, round, table1, table2, true);
                        self.current_score = self.next_score;
                        if self.current_score > self.best_score
                            || (self.current_score == self.best_score
                                && self.current.unique_games_played()
                                    > self.best.unique_games_played())
                        {
                            self.best_score = self.current_score;
                            self.best = self.current.clone();
                        }
                    } else {
                        self.current = Schedule::new(self.player_count, self.tables);
                        self.current.generate_random(&mut self.rng);
                        self.current_score = self.current.generate_score();
                        self.next_score = self.current_score;
                        if self.current_score > self.best_score
                            || (self.current_score == self.best_score
                                && self.current.unique_games_played()
                                    > self.best.unique_games_played())
                        {
                            self.best_score = self.current_score;
                            self.best = self.current.clone();
                        }
                    }
                }
            }
        }
        let (new_score, new_unique_games_played) = self.current.improve_table(
            self.current_score,
            self.round,
            self.table1,
            self.table2,
            false,
        );
        if new_score > self.next_score
            || (new_score == self.next_score && new_unique_games_played > self.next_unique_games)
        {
            self.next = (self.round, self.table1, self.table2);
            self.next_score = new_score;
            self.next_unique_games = new_unique_games_played;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_games_played_returns_number_of_player_when_players_stay() {
        let round: Vec<Vec<usize>> = vec![
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8, 9, 10, 11],
            vec![12, 13, 14, 15],
            vec![16, 17, 18, 19],
            vec![20, 21, 22, 23],
        ];
        let mut game: Vec<Vec<Vec<usize>>> = Vec::new();
        for _ in 0..6 {
            game.push(round.clone());
        }
        let mut schedule = Schedule::from_vec(24, 6, game);
        assert_eq!(24, schedule.find_unique_games_played());
    }

    #[test]
    fn unique_opponents_is_team_size_when_players_stay() {
        let round: Vec<Vec<usize>> = vec![
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8, 9, 10, 11],
            vec![12, 13, 14, 15],
            vec![16, 17, 18, 19],
            vec![20, 21, 22, 23],
        ];
        let mut game: Vec<Vec<Vec<usize>>> = Vec::new();
        for _ in 0..6 {
            game.push(round.clone());
        }
        let mut schedule = Schedule::from_vec(24, 6, game);
        assert_eq!(3 * 24, schedule.find_unique_opponents());
    }
    #[derive(Clone, Debug)]
    struct Seed {
        pub data: [u8; 16],
    }

    impl quickcheck::Arbitrary for Seed {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let mut data: [u8; 16] = [0; 16];
            for i in 0..16 {
                data[i] = u8::arbitrary(g);
            }
            Seed { data }
        }
    }

    quickcheck! {fn get_score_matches_generate_score(tables: i8, player_count: i8, seed: Seed) -> bool{
        let tables = (tables.abs() & 63).max(2) as usize;
        let player_count = (player_count.abs() & 63) as usize;
        let mut schedule = Schedule::new(player_count as usize, tables as usize);
        let mut rng = rand_xorshift::XorShiftRng::from_seed(seed.data);
        schedule.generate_random(&mut rng);
        schedule.get_score() == schedule.generate_score()
    }}

    quickcheck! {fn unique_games_played_less_equal_players_times_games_before_process(tables: i8, player_count: i8, seed: Seed) -> bool{
        let tables = (tables.abs() & 63).max(2) as usize;
        let player_count = (player_count.abs() & 63) as usize;
        let mut schedule = Schedule::new(player_count as usize, tables as usize);
        let mut rng = rand_xorshift::XorShiftRng::from_seed(seed.data);
        schedule.generate_random(&mut rng);
        schedule.find_unique_games_played() <= (tables * player_count) as u32
    }}

    quickcheck! {fn score_doesnt_decrease_after_process(tables: i8, player_count: i8, seed: Seed) -> bool{
        let tables = (tables.abs() & 63).max(2) as usize;
        let player_count = (player_count.abs() & 63) as usize;
        let rng = rand_xorshift::XorShiftRng::from_seed(seed.data);
        let mut generator = Generator::new(rng, player_count as usize, tables as usize);
    let old_score = generator.best_score;
        generator.process();
        generator.best_score >= old_score
    }}
}
