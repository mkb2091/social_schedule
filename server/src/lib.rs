pub mod ui_pages;

use std::sync::Mutex;

pub use seed;
pub use warp;

pub struct State {
    pub scheduler: Mutex<(Vec<usize>, usize)>,
}
