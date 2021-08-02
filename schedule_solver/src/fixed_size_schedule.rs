use crate::scheduler::*;
use crate::word::Word;

#[derive(Debug)]
pub struct FixedSchedule<
    T: Word,
    const A: usize,
    const B: usize,
    const C: usize,
    const D: usize,
    const E: usize,
    const F: usize,
> {
    to_explore: [T; A],
    played_with: [T; B],
    played_on_table: [T; C],
    played_in_round: [T; D],
    played_in_game: [T; E],
    potential_in_game: [T; F],
}

impl<
        T: Word,
        const A: usize,
        const B: usize,
        const C: usize,
        const D: usize,
        const E: usize,
        const F: usize,
    > Schedule<T> for FixedSchedule<T, A, B, C, D, E, F>
{
    fn new(sizes: Sizes) -> Option<Self> {
        if sizes.get_to_explore_size() > A
            || sizes.get_played_with_size() > B
            || sizes.get_played_on_table_size() > C
            || sizes.get_played_in_round_size() > D
            || sizes.get_played_in_game_size() > E
            || sizes.get_potential_in_game_size() > F
        {
            None
        } else {
            Some(Self {
                to_explore: [T::ZERO; A],
                played_with: [T::ZERO; B],
                played_on_table: [T::ZERO; C],
                played_in_round: [T::ZERO; D],
                played_in_game: [T::ZERO; E],
                potential_in_game: [T::ZERO; F],
            })
        }
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
