pub mod board;
pub mod finish;
pub mod revenge;
pub mod siege;
pub mod target;

pub use finish::{battle_finish, raid_attack};
pub use target::{battle_log, find_target};
