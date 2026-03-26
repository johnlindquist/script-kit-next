#![allow(dead_code)]

mod tokens;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChromeStyle {
    #[default]
    Minimal,
    Rich,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowMode {
    Mini,
    #[default]
    Standard,
    Full,
}

pub use tokens::*;
