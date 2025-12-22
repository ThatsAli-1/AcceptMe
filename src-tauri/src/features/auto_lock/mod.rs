mod logic;
mod commands;

pub use logic::execute_auto_lock;
pub use commands::{set_auto_select, get_auto_select};
