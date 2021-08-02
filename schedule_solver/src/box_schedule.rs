use crate::scheduler::*;
use crate::word::Word;

#[derive(Debug)]
pub struct BoxSchedule<T: Word> {
    to_explore: Box<[T]>,
    played_with: Box<[T]>,
    played_on_table: Box<[T]>,
    played_in_round: Box<[T]>,
    played_in_game: Box<[T]>,
    potential_in_game: Box<[T]>,
}

impl<T: Word> Schedule<T> for BoxSchedule<T> {
    fn new(sizes: Sizes) -> Option<Self> {
        Some(Self {
            to_explore: vec![T::ZERO; sizes.get_to_explore_size()].into_boxed_slice(),
            played_with: vec![T::ZERO; sizes.get_played_with_size()].into_boxed_slice(),
            played_on_table: vec![T::ZERO; sizes.get_played_on_table_size()].into_boxed_slice(),
            played_in_round: vec![T::ZERO; sizes.get_played_in_round_size()].into_boxed_slice(),
            played_in_game: vec![T::ZERO; sizes.get_played_in_game_size()].into_boxed_slice(),
            potential_in_game: vec![T::ZERO; sizes.get_potential_in_game_size()].into_boxed_slice(),
        })
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
