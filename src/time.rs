use derive_more::{Add, AddAssign, Display, From, Into, Sub, Sum};

/// This library uses a simple discrete time model.
///
/// We do not use unchecked arithmetic, so any over- or underflow
/// will be detected by the Rust runtime system.
pub type Time = u64;

/// Type-safe alias for time value representing an offset.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into, Display, Debug)]
pub struct Offset {
    val: Time,
}

impl Offset {
    /// Given a duration `delta`, returns an offset `X` such that the
    /// half-open interval `[0, X)` has length `delta`.
    pub const fn from_time_zero(delta: Duration) -> Offset {
        Offset { val: delta.val }
    }

    /// Given a duration `delta`, returns an offset `X` such that the
    /// closed interval `[0, X]` has length `delta`.
    pub const fn closed_from_time_zero(delta: Duration) -> Offset {
        Offset { val: delta.val - 1 }
    }

    /// Given an offset `X`, returns the length of the half-open
    /// interval `[0, X)`.
    ///
    /// # Example
    /// ```
    /// # use response_time_analysis::time::Offset;
    /// let t = Offset::from(10);
    /// assert_eq!(Offset::from_time_zero(t.since_time_zero()), t);
    /// ```
    pub const fn since_time_zero(self) -> Duration {
        Duration { val: self.val }
    }

    /// Given an offset `X`, returns the length of the closed
    /// interval `[0, X]`.
    ///
    /// # Example
    /// ```
    /// # use response_time_analysis::time::Offset;
    /// let t = Offset::from(10);
    /// assert_eq!(Offset::closed_from_time_zero(t.closed_since_time_zero()), t);
    /// ```
    pub const fn closed_since_time_zero(self) -> Duration {
        Duration { val: self.val + 1 }
    }

    /// Compute the distance to a later point in time.
    ///
    /// # Example:
    ///
    /// ```
    /// # use response_time_analysis::time::Offset;
    /// let a = Offset::from(10);
    /// let b = Offset::from(25);
    /// assert_eq!(a + a.distance_to(b), b);
    /// ```
    pub fn distance_to(self, t: Offset) -> Duration {
        debug_assert!(self.val <= t.val);
        Duration::from(t.val - self.val)
    }
}

impl std::ops::Add<Duration> for Offset {
    type Output = Offset;

    fn add(self, delta: Duration) -> Offset {
        Offset {
            val: self.val + delta.val,
        }
    }
}

/// Type-safe alias for time values representing an interval length.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
    Display,
    Debug,
    Add,
    AddAssign,
    Sub,
    Sum,
)]
pub struct Duration {
    val: Time,
}

impl Duration {
    /// Check whether the duration represents a non-empty interval.
    pub const fn is_non_zero(self) -> bool {
        self.val > 0
    }

    /// Check whether the duration represents an empty interval.
    pub const fn is_zero(self) -> bool {
        self.val == 0
    }

    /// Construct a duration value that represents the empty interval.
    pub const fn zero() -> Duration {
        Duration { val: 0 }
    }

    /// Construct a duration value that represents the smallest unit
    /// of time (i.e., 1).
    pub const fn epsilon() -> Duration {
        Duration { val: 1 }
    }

    /// Subtract without under-flowing.
    #[must_use]
    pub const fn saturating_sub(&self, rhs: Duration) -> Duration {
        Duration {
            val: self.val.saturating_sub(rhs.val),
        }
    }
}

impl From<Service> for Duration {
    fn from(s: Service) -> Duration {
        Duration::from(s.val)
    }
}

impl std::ops::Mul<u64> for Duration {
    type Output = Duration;

    fn mul(self, factor: u64) -> Duration {
        Duration::from(self.val * factor)
    }
}

impl std::ops::Div<Duration> for Duration {
    type Output = u64;

    fn div(self, divisor: Duration) -> u64 {
        self.val / divisor.val
    }
}

impl std::ops::Rem<Duration> for Duration {
    type Output = Duration;

    fn rem(self, divisor: Duration) -> Duration {
        Duration::from(self.val % divisor.val)
    }
}

/// Type-safe alias for time values representing some amount of
/// processor service.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
    Display,
    Debug,
    Add,
    Sub,
    AddAssign,
    Sum,
)]
pub struct Service {
    val: Time,
}

impl Service {
    /// Construct a service value that represents zero service.
    pub fn none() -> Service {
        Service::from(0)
    }

    /// Check whether the value represents zero service.
    pub fn is_none(self) -> bool {
        self.val == 0
    }

    /// The amount of service received in interval of duration `d`
    /// (assuming a unit-service processor).
    pub const fn in_interval(d: Duration) -> Service {
        Service { val: d.val }
    }

    /// The least unit of service.
    pub const fn epsilon() -> Service {
        Service { val: 1 }
    }

    /// Subtract without under-flowing.
    #[must_use]
    pub const fn saturating_sub(&self, rhs: Service) -> Service {
        Service {
            val: self.val.saturating_sub(rhs.val),
        }
    }
}

impl From<Duration> for Service {
    fn from(d: Duration) -> Service {
        Service::in_interval(d)
    }
}

impl std::ops::Mul<u64> for Service {
    type Output = Service;

    fn mul(self, factor: u64) -> Service {
        Service::from(self.val * factor)
    }
}
