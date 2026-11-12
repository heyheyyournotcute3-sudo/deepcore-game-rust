//! Crafting and machine processing rules, including the mining break-time formula.

pub mod mining;

pub use mining::{MineCheck, break_time, check_mine, mine_seconds, ticks_for};
