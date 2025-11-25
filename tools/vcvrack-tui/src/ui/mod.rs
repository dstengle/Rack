//! User interface module

pub mod completion;
pub mod input;
pub mod render;

pub use completion::CompletionEngine;
pub use input::{handle_event, InputResult};
pub use render::render;
