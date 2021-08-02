use crate::scheduler::*;
use crate::word::Word;

#[derive(Debug)]
pub struct RefSchedule<'a, T: Word> {
    to_explore: &'a mut [T],
    played_with: &'a mut [T],
    played_on_table: &'a mut [T],
    played_in_round: &'a mut [T],
    played_in_game: &'a mut [T],
    potential_in_game: &'a mut [T],
}

impl<'a, T: Word> RefSchedule<'a, T> {
    pub fn from_slice(sizes: Sizes, slice: &'a mut [T]) -> Option<Self> {
        if slice.len() < sizes.get_total_size() {
            None
        } else {
            let (to_explore, slice) = slice.split_at_mut(sizes.get_to_explore_size());
            let (played_with, slice) = slice.split_at_mut(sizes.get_played_with_size());
            let (played_on_table, slice) = slice.split_at_mut(sizes.get_played_on_table_size());
            let (played_in_round, slice) = slice.split_at_mut(sizes.get_played_in_round_size());
            let (played_in_game, slice) = slice.split_at_mut(sizes.get_played_in_game_size());
            let (potential_in_game, slice) = slice.split_at_mut(sizes.get_potential_in_game_size());
            Some(Self {
                to_explore,
                played_with,
                played_on_table,
                played_in_round,
                played_in_game,
                potential_in_game,
            })
        }
    }
}

impl<'a, T: Word> Schedule<T> for RefSchedule<'a, T> {
    fn new(sizes: Sizes) -> Option<Self> {
        None
    }
    fn get_to_explore(&self) -> &[T] {
        &self.to_explore
    }
    fn get_to_explore_mut(&mut self) -> &mut [T] {
        &mut self.to_explore
    }
    fn get_played_with(&self) -> &[T] {
        &self.played_with
    }
    fn get_played_with_mut(&mut self) -> &mut [T] {
        &mut self.played_with
    }
    fn get_played_on_table(&self) -> &[T] {
        &self.played_on_table
    }
    fn get_played_on_table_mut(&mut self) -> &mut [T] {
        &mut self.played_on_table
    }
    fn get_played_in_round(&self) -> &[T] {
        &self.played_in_round
    }
    fn get_played_in_round_mut(&mut self) -> &mut [T] {
        &mut self.played_in_round
    }
    fn get_played_in_game(&self) -> &[T] {
        &self.played_in_game
    }
    fn get_played_in_game_mut(&mut self) -> &mut [T] {
        &mut self.played_in_game
    }
    fn get_potential_in_game(&self) -> &[T] {
        &self.potential_in_game
    }
    fn get_potential_in_game_mut(&mut self) -> &mut [T] {
        &mut self.potential_in_game
    }
}
