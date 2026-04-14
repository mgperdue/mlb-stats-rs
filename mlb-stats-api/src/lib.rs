// mlb-stats-api/src/lib.rs
//
// Public API surface for the `mlb-stats-api` crate.
//
// All response types are accessible via `mlb_stats_api::models::*` or
// through the flat re-exports below.  The `MlbClient` is the primary
// entry-point for making requests.
//
// # Feature flags
//
// - `test-utils` — enables [`mock::MockMlbClient`], a fixture-backed mock
//   of `MlbClient` for use in consumer test suites.  Enable it only in
//   `[dev-dependencies]` or behind `#[cfg(test)]`.
//
// # Example
//
// ```rust,no_run
// use mlb_stats_api::MlbClient;
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let client = MlbClient::new();
//     let schedule = client.get_schedule_today().await?;
//     println!("{} games today", schedule.total_games.unwrap_or(0));
//     Ok(())
// }
// ```

pub mod client;
pub mod error;
pub mod models;

// The mock module is compiled only when the `test-utils` feature is active.
// Downstream consumers add `mlb-stats-api = { features = ["test-utils"] }`
// in their `[dev-dependencies]` block.
#[cfg(feature = "test-utils")]
pub mod mock;

// ---------------------------------------------------------------------------
// Flat re-exports for ergonomic imports
// ---------------------------------------------------------------------------

pub use client::MlbClient;
pub use error::MlbApiError;

// Core ID newtypes — re-exported so consumers can write
// `mlb_stats_api::TeamId` without knowing the models sub-path.
pub use models::common::{DivisionId, LeagueId, TeamId};

// Meta types re-exported at crate root — matches Python statsapi.meta() UX.
pub use models::meta::{MetaEntry, MetaType};