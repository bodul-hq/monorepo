//! Infrastructure adapters for time and date types.
//!
//! This module provides infrastructure wrappers for momentum types,
//! enabling serialization and integration with external libraries.

mod date_time;

pub mod prelude {
    //! Commonly used types from the momentum infrastructure module.

    pub use super::date_time::AppDateTime;
}
