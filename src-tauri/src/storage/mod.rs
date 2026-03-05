//! SQLite persistence layer for 900CRM.
//!
//! This module provides database access for all CRM entities. It exports the
//! [`Database`] struct (re-exported here for convenience) and organises CRUD
//! operations into focused sub-modules:
//!
//! | Sub-module   | Responsibility |
//! |--------------|----------------|
//! | [`db`]       | Connection setup, PRAGMAs, schema migrations |
//! | [`contacts`] | Contact CRUD + FTS5 search |
//! | [`deals`]    | Deal CRUD + pipeline stage operations |
//! | [`activities`] | Activity CRUD + scheduling queries |
//! | [`notes`]    | Entity-attached notes CRUD |
//! | [`tags`]     | Tag management and entity tagging |
//! | [`settings`] | Key-value application settings |
//! | [`sync`]     | Offline-first sync changelog |
//! | [`reporting`]| Reporting aggregates for funnel/conversion analytics |
//!
//! All storage functions accept a `&rusqlite::Connection` reference (obtained
//! from `db.conn`) so they can be composed without extra locking overhead.

pub mod activities;
pub mod contacts;
pub mod custom_fields;
pub mod db;
pub mod deals;
pub mod notes;
pub mod reporting;
pub mod settings;
pub mod sync;
pub mod tags;

/// Re-export the [`Database`] struct at the storage module level.
///
/// Command handlers and [`crate::AppState`] use `storage::Database` without
/// needing to descend into `storage::db::Database`.
pub use db::Database;
