//! forgetter: a derivation log, and the lineage read back out of it.
//!
//! A derivation records one execution: an action, the artifacts it
//! consumed, and the artifacts it produced, each named by its `cas`
//! digest rather than embedded. What an action computes, and where it
//! keeps its own state, this crate never looks at.
//!
//! Neither type is built yet.

/// Appends derivations. Owns a local directory and the task writing into
/// it, so there is one writer.
pub struct Log;

/// Traces the log back as a graph of derivations. Needs only the object
/// store a [`Log`] publishes to, so it can run anywhere.
pub struct Lineage;
