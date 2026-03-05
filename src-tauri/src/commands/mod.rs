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
//! | [`custom_field_commands`] | Custom field definitions and values |
//! | [`report_commands`]   | Reporting metrics (pipeline + activity funnels) |
//! | [`import_export`]     | CSV import/export via file dialog |
//! | [`settings_commands`] | Application preferences |
//! | [`email_commands`]    | Optional IMAP/SMTP connection checks |

pub mod activity_commands;
pub mod contact_commands;
pub mod custom_field_commands;
pub mod dashboard_commands;
pub mod deal_commands;
pub mod email_commands;
pub mod import_export;
pub mod report_commands;
pub mod settings_commands;
pub mod sync_commands;
