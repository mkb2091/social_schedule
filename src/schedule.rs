use rand::seq::SliceRandom;
use std::ops::IndexMut;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Schedule {
    player_count: usize,
    tables: usize,
    matches: Vec<u64>,
    score_multiplier: u32,
}

impl Schedule {
    pub fn new(player_count: usize, tables: usize) -> Schedule {
        let mut matches: Vec<u64> = Vec::with_capacity(tables * tables);
        for _ in 0..(tables * tables) {
            matches.push(0);
        }
        Schedule {
            player_count,
            tables,
            matches,
            score_multiplier: (player_count.pow(2) - player_count * tables) as u32,
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
        for (round_number, round) in data.iter().enumerate() {
            for (table_number, table) in round.iter().enumerate() {
                for player in table.iter() {
                    *self.get_mut(round_number, table_number) |= 2_u64.pow(*player as u32);
                }
            }
        }
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

    pub fn unique_games_played(&self) -> u32 {
        let mut total: u32 = 0;

        for table in 0..self.tables {
            let mut total_table = 0;
            for round in 0..self.tables {
                total_table |= self.get(round, table);
            }
            total += total_table.count_ones();
        }
        total
    }

    pub fn unique_opponents(&self) -> u32 {
        let mut total: u32 = 0;
        for player in 0..self.player_count {
            let mut opponents: u64 = 0;
            let player = 1 << player;
            for round in 0..self.tables {
                for table in 0..self.tables {
                    let game = self.get(round, table);
                    if game & player != 0 {
                        opponents |= game;
                        break;
                    }
                }
            }
            total += opponents.count_ones();
        }
        total - self.player_count as u32
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
    pub fn generate_score(&self) -> u32 {
        self.unique_opponents() + self.unique_games_played() * self.score_multiplier
    }
    pub fn improve_table(
        &mut self,
        mut score: u32,
        round: usize,
        table1: usize,
        table2: usize,
    ) -> bool {
        let original_t1 = self.get(round, table1);
        let original_t2 = self.get(round, table2);
        let mut best_t1 = original_t1;
        let mut best_t2 = original_t2;
        let mut changed = false;
        for player1 in 0..self.player_count {
            let player_number1 = 1 << player1;
            if original_t1 & player_number1 != 0 {
                for player2 in 0..self.player_count {
                    let player_number2 = 1 << player2;
                    if original_t2 & player_number2 != 0 {
                        *self.get_mut(round, table1) =
                            original_t1 - player_number1 + player_number2;
                        *self.get_mut(round, table2) =
                            original_t2 - player_number2 + player_number1;
                        let new_score = self.generate_score();
                        if new_score > score {
                            changed = true;
                            best_t1 = original_t1 - player_number1 + player_number2;
                            best_t2 = original_t2 - player_number2 + player_number1;
                            score = new_score;
                        }
                    }
                }
            }
        }
        *self.get_mut(round, table1) = best_t1;
        *self.get_mut(round, table2) = best_t2;
        changed
    }
}

pub struct ScheduleGenerator<T: rand::Rng + rand_core::RngCore> {
    player_count: usize,
    tables: usize,
    pub best: Schedule,
    pub best_score: u32,
    current: Schedule,
    current_score: u32,
    rng: T,
}

impl<T: rand::Rng + rand_core::RngCore> ScheduleGenerator<T> {
    pub fn new(mut rng: T, player_count: usize, tables: usize) -> ScheduleGenerator<T> {
        let mut best = Schedule::new(player_count, tables);
        best.generate_random(&mut rng);
        let score = best.generate_score();
        ScheduleGenerator {
            player_count,
            tables,
            best: best.clone(),
            best_score: score,
            current: best,
            current_score: score,
            rng,
        }
    }

    pub fn process(&mut self) {
        for round in 0..self.tables {
            for table1 in 0..self.tables {
                for table2 in 0..self.tables {
                    if table1 == table2 {
                        continue;
                    }
                    if self
                        .current
                        .improve_table(self.current_score, round, table1, table2)
                    {
                        self.current_score = self.current.generate_score();
                        if self.current_score > self.best_score {
                            self.best_score = self.current_score;
                            self.best = self.current.clone();
                        }
                        return;
                    }
                }
            }
        }
        self.current = Schedule::new(self.player_count, self.tables);
        self.current.generate_random(&mut self.rng);
        if self.current.generate_score() > self.best_score {
            self.best_score = self.current.generate_score();
            self.best = self.current.clone();
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
        let schedule = Schedule::from_vec(24, 6, game);
        assert_eq!(24, schedule.unique_games_played());
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
        let schedule = Schedule::from_vec(24, 6, game);
        assert_eq!(3 * 24, schedule.unique_opponents());
    }
}
