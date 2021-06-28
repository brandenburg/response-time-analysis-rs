/// This library uses a simple discrete time model.
pub type Time = u64;

/// Syntactic sugar to give a hint that a time value indicates a
/// point in time or some offset.
pub type Instant = Time;

/// Syntactic sugar to give a hint that a time value denotes an
/// interval length.
pub type Duration = Time;

/// Syntactic sugar to give a hint that a time value represents some
/// amount of processor service.
pub type Service = Time;
