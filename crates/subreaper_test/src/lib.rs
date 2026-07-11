//! Provides helper function(s) for subreaper testing.

use std::path::PathBuf;

use runfiles::{
    Runfiles,
    rlocation,
};

/// Returns the path to the executable `orphan`.
pub fn orphan() -> PathBuf {
    let r = Runfiles::create().expect("failed to create Runfiles");
    let path = rlocation!(r, "_main/crates/subreaper_test/orphan")
        .expect("failed to resolve the orphan runfile");
    assert!(path.exists());
    path
}
