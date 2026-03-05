//! Tauri IPC command handlers for 900CRM.
//!
//! This module declares all sub-modules containing `#[tauri::command]`
//! functions. Every command is registered in the `invoke_handler` inside
//! [`crate::run`].
//!
//! # Module Overview
//!
//! | Sub-module            | Commands |
//! |-----------------------|----------|
//! | [`contact_commands`]  | Contact CRUD, search, merge |
//! | [`deal_commands`]     | Deal CRUD, pipeline management |
//! | [`activity_commands`] | Activity CRUD, scheduling |
//! | [`dashboard_commands`]| Aggregate statistics |
//! | [`import_export`]     | CSV import/export via file dialog |
//! | [`settings_commands`] | Application preferences |

pub mod activity_commands;
pub mod contact_commands;
pub mod dashboard_commands;
pub mod deal_commands;
pub mod import_export;
pub mod settings_commands;
