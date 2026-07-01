// Integration test crate for Soundome.
// Each module targets a specific package or external integration.
// Tests that hit real external APIs are gated with #[ignore].
//
// Usage:
//   cargo test -p soundome-tests                       # unit-only (always run)
//   cargo test -p soundome-tests -- --ignored          # all ignored (live API) tests
//   cargo test -p soundome-tests -- --ignored --nocapture  # with log output

pub mod proxy_rotation;
pub mod spotify;
