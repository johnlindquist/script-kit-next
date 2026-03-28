//! Windows stub for window_control module
//!
//! Provides the same API as the macOS window_control module but returns
//! "not implemented" errors. This allows the code to compile on Windows
//! without conditionally compiling all window control command handlers.

// This module is Windows-only (inverse of the real window_control module)
#![cfg(not(target_os = "macos"))]

use anyhow::{anyhow, Result};

/// Stub Bounds type matching window_control::Bounds
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Stub WindowInfo type matching window_control::WindowInfo
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct WindowInfo {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub bounds: Bounds,
    pub pid: i32,
}

/// Stub TilePosition matching window_control::TilePosition
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum TilePosition {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    LeftThird,
    CenterThird,
    RightThird,
    TopThird,
    MiddleThird,
    BottomThird,
    FirstTwoThirds,
    LastTwoThirds,
    TopTwoThirds,
    BottomTwoThirds,
    Center,
    AlmostMaximize,
    Fullscreen,
}

// Stub implementations of all window_control functions

pub fn list_windows() -> Result<Vec<WindowInfo>> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn focus_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn close_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn minimize_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn maximize_window(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn resize_window(_id: u32, _width: u32, _height: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn move_window(_id: u32, _x: i32, _y: i32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn tile_window(_id: u32, _position: TilePosition) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn move_to_next_display(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn move_to_previous_display(_id: u32) -> Result<()> {
    Err(anyhow!("Window control not implemented on Windows"))
}

pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>> {
    Err(anyhow!("Window control not implemented on Windows"))
}
