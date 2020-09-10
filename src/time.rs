// We use a simple discrete time model
pub type Time = u64;

// Syntactic sugar to give a hint as whether a time value denotes an interval length
// or a specific point in time.
pub type Instant = Time;
pub type Duration = Time;
