//! Infrastructure implementations for everyday concepts
//!
//! This library provides infrastructure layer implementations for everyday types,
//! including:
//! - Database models for time, weight, distance, currency, and country codes
//! - Serialization and deserialization for persistence
//! - Storage adapters and repositories
//! - Type conversions for external APIs and databases
//! - Integration with external systems and services
//!
//! This crate depends on the `everyday` crate and provides the infrastructure
//! layer implementations needed to store and retrieve everyday concepts.

pub mod business;
pub mod momentum;
pub mod prelude;
