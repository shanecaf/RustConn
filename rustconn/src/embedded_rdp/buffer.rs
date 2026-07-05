//! Render buffer for embedded RDP.
//!
//! Re-exports `CairoBackedBuffer` (the zero-copy render surface used by the
//! IronRDP embedded path). The former placeholder `WaylandSurfaceHandle` and
//! the legacy `PixelBuffer` were removed in 0.18.0 as dead code.

/// A pixel buffer backed by a persistent Cairo `ImageSurface`.
///
/// Instead of cloning 33MB of pixel data on every draw call (at 4K),
/// this struct owns the underlying byte buffer via Cairo's
/// `ImageSurface::create_for_data()` and provides mutable access
/// through `surface.data()` for in-place updates.
///
/// Re-exported from [`crate::cairo_buffer::CairoBackedBuffer`].
pub use crate::cairo_buffer::CairoBackedBuffer;
