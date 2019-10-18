use std::ops::IndexMut;

pub struct Schedule {
    player_count: usize,
    tables: usize,
    matches: Vec<u64>,
}

impl Schedule {
    pub fn new(player_count: usize, tables: usize) -> Result<Schedule, ()> {
        if tables > player_count {
            return Err(());
        } else {
            let mut matches: Vec<u64> = Vec::with_capacity(tables * tables);
            for _ in 0..(tables * tables) {
                matches.push(0);
            }

            Ok(Schedule {
                player_count,
                tables,
                matches,
            })
        }
    }

    pub fn from_vec(
        player_count: usize,
        tables: usize,
        data: Vec<Vec<Vec<usize>>>,
    ) -> Result<Schedule, ()> {
        if let Ok(mut new) = Schedule::new(player_count, tables) {
            for (round_number, round) in data.iter().enumerate() {
                for (table_number, table) in round.iter().enumerate() {
                    for player in table.iter() {
                        *new.get_mut(round_number, table_number) |= (2_u64.pow(*player as u32));
                    }
                }
            }
            Ok(new)
        } else {
            Err(())
        }
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
        0
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
        if let Ok(mut schedule) = Schedule::from_vec(24, 6, game) {
            assert_eq!(24, schedule.unique_games_played());
        }
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
        if let Ok(mut schedule) = Schedule::from_vec(24, 6, game) {
            assert_eq!(4, schedule.unique_opponents());
        }
    }
}
