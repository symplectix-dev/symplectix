//! Provides helper function(s) for subreaper testing.

use std::path::PathBuf;

/// Returns the path to the executable `orphan`.
pub fn orphan() -> PathBuf {
    let path = testing::rlocation("_main/crates/subreaper_test/orphan");
    assert!(path.exists());
    path
}
