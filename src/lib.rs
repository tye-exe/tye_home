#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod js_imports;
mod logger;

pub use app::MyApp;
pub use logger::{Logger, Transmitted as LogType};
