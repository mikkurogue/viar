//! QMK firmware settings definitions and types.
//!
//! This crate provides a complete mapping of QMK Settings IDs as defined in
//! the Vial protocol (`qmk_settings.c` / `qmk_settings.h` in vial-qmk).
//!
//! Settings are organized into categories that mirror the Vial web interface.
//! Each setting has an ID, a type (u8/u16/u32 or bitfield), a human-readable
//! name, and metadata for UI rendering (range, description, category).

mod settings;

pub use settings::*;
