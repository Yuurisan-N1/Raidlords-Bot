pub mod board;
pub mod finish;
pub mod revenge;
pub mod siege;
pub mod target;

pub use finish::battle_finish;
pub use target::{battle_log, find_target, next_target};
