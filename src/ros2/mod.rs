mod ecrts19;

/// Busy-window-aware analysis that exploits both the non-starvation
/// property of the ROS2 callback scheduler and the busy-window
/// principle.
pub mod bw;
/// Round-robin-aware analysis that exploits the non-starvation
/// property of the ROS2 callback scheduler.
pub mod rr;

pub use ecrts19::{rta_event_source, rta_polling_point_callback, rta_processing_chain, rta_timer};

#[cfg(test)]
mod tests;
