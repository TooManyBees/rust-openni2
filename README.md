# OpenNI2 for Rust

In-development Rust wrapper for [OpenNI2](https://github.com/occipital/OpenNI2).
OpenNI2 is useful for working with multi-sensor cameras that can simultaneously
serve color and depth streams, particularly sensors developed by PrimeSense
(a founding member of the OpenNI software project) such as the Xbox Kinect,
and ASUS Xtion.

# Development goals

Still feeling out a comfortable API that conforms to Rust's ergonomics.

# Examples

[`examples/data_dump.rs`](examples/data_dump.rs) demonstrates interrogating
devices and streams about their properties, as well as blocking for new frames.

[`examples/closest_point.rs`](examples/closest_point.rs) demonstrates event-based
callbacks, and finding the closest point in a depth map.
