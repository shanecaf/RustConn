//! Embedded SPICE widget for native SPICE session embedding
//!
//! This module provides the `EmbeddedSpiceWidget` struct that enables native
//! SPICE session embedding within GTK4 applications using the `spice-client` crate.
//!
//! # Architecture
//!
//! The widget uses a `DrawingArea` for rendering SPICE frames and handles:
//! - Connection lifecycle management
//! - Framebuffer rendering from SPICE client events
//! - Keyboard and mouse input forwarding
//! - Fallback to external viewer (remote-viewer) when native fails

use crate::i18n::i18n;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DrawingArea, Label, Orientation};
use std::cell::RefCell;
use std::process::Child;
use std::rc::Rc;

use rustconn_core::spice_client::{SpiceClientConfig, SpiceClientError};

/// Connection state for embedded SPICE widget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpiceConnectionState {
    /// Not connected
    #[default]
    Disconnected,
    /// Connection in progress
    Connecting,
    /// Connected and rendering
    Connected,
    /// Connection error occurred
    Error,
}

impl std::fmt::Display for SpiceConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Pixel buffer for SPICE frame data
#[derive(Debug)]
pub struct SpicePixelBuffer {
    /// Raw pixel data in BGRA format
    data: Vec<u8>,
    /// Buffer width in pixels
    width: u32,
    /// Buffer height in pixels
    height: u32,
    /// Stride (bytes per row)
    stride: u32,
}

impl SpicePixelBuffer {
    /// Creates a new pixel buffer with the specified dimensions
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let stride = width * 4; // 4 bytes per pixel (BGRA)
        let size = (stride * height) as usize;
        Self {
            data: vec![0; size],
            width,
            height,
            stride,
        }
    }

    /// Returns the buffer width
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns the buffer height
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns a reference to the raw pixel data
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Resizes the buffer to new dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.stride = width * 4;
        let size = (self.stride * height) as usize;
        self.data.resize(size, 0);
    }

    /// Clears the buffer to black
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Updates a region of the buffer with raw pixel data
    pub fn update_region(
        &mut self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        src_data: &[u8],
        src_stride: u32,
    ) {
        let dst_stride = self.stride as usize;
        let src_stride = src_stride as usize;
        let bytes_per_pixel = 4;

        for row in 0..h {
            let dst_y = (y + row) as usize;
            if dst_y >= self.height as usize {
                break;
            }

            let dst_offset = dst_y * dst_stride + (x as usize * bytes_per_pixel);
            let src_offset = row as usize * src_stride;
            let copy_width =
                (w as usize * bytes_per_pixel).min(dst_stride - (x as usize * bytes_per_pixel));

            if src_offset + copy_width <= src_data.len()
                && dst_offset + copy_width <= self.data.len()
            {
                self.data[dst_offset..dst_offset + copy_width]
                    .copy_from_slice(&src_data[src_offset..src_offset + copy_width]);
            }
        }
    }
}

/// Callback type for state change notifications
type StateCallback = Box<dyn Fn(SpiceConnectionState) + 'static>;

/// Callback type for error notifications
type ErrorCallback = Box<dyn Fn(&str) + 'static>;

/// Embedded SPICE widget using native spice-client
///
/// This widget provides native SPICE session embedding within GTK4 applications.
/// It uses a `DrawingArea` for rendering and integrates with the SPICE client
/// from `rustconn-core`.
pub struct EmbeddedSpiceWidget {
    /// Main container widget
    container: GtkBox,
    /// Toolbar with clipboard and special key buttons
    toolbar: GtkBox,
    /// Status label for feedback
    #[expect(
        dead_code,
        reason = "part of the toolbar UI; transient status text was only updated by the removed native path"
    )]
    status_label: Label,
    /// Copy button
    copy_button: Button,
    /// Paste button
    paste_button: Button,
    /// Ctrl+Alt+Del button
    ctrl_alt_del_button: Button,
    /// Separator between buttons
    separator: gtk4::Separator,
    /// Drawing area for rendering SPICE frames
    drawing_area: DrawingArea,
    /// Pixel buffer for frame data
    pixel_buffer: Rc<RefCell<SpicePixelBuffer>>,
    /// Persistent Cairo-backed pixel buffer for zero-copy rendering
    cairo_buffer: Rc<RefCell<crate::cairo_buffer::CairoBackedBuffer>>,
    /// Current connection state
    state: Rc<RefCell<SpiceConnectionState>>,
    /// Current configuration
    config: Rc<RefCell<Option<SpiceClientConfig>>>,
    /// External viewer child process (for fallback mode)
    process: Rc<RefCell<Option<Child>>>,
    /// Whether using embedded mode or external mode
    is_embedded: Rc<RefCell<bool>>,
    /// Current widget width
    width: Rc<RefCell<u32>>,
    /// Current widget height
    height: Rc<RefCell<u32>>,
    /// SPICE server framebuffer width
    #[expect(
        dead_code,
        reason = "tracked the remote framebuffer size for the removed native renderer"
    )]
    spice_width: Rc<RefCell<u32>>,
    /// SPICE server framebuffer height
    #[expect(
        dead_code,
        reason = "tracked the remote framebuffer size for the removed native renderer"
    )]
    spice_height: Rc<RefCell<u32>>,
    /// State change callback
    on_state_changed: Rc<RefCell<Option<StateCallback>>>,
    /// Error callback
    on_error: Rc<RefCell<Option<ErrorCallback>>>,
    /// Reconnect callback
    on_reconnect: Rc<RefCell<Option<Box<dyn Fn() + 'static>>>>,
    /// Reconnect banner (shown when disconnected, at bottom of container)
    reconnect_banner: GtkBox,
    /// Reconnect button inside the banner
    reconnect_button: Button,
}

impl EmbeddedSpiceWidget {
    /// Creates a new embedded SPICE widget
    #[must_use]
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        // Create toolbar with clipboard and Ctrl+Alt+Del buttons
        let toolbar = GtkBox::new(Orientation::Horizontal, 4);
        toolbar.add_css_class("embedded-toolbar");
        toolbar.set_margin_start(6);
        toolbar.set_margin_end(6);
        toolbar.set_margin_top(6);
        toolbar.set_margin_bottom(6);
        toolbar.set_halign(gtk4::Align::End);

        // Status label for feedback
        let status_label = Label::new(None);
        status_label.set_visible(false);
        status_label.set_margin_end(12);
        status_label.add_css_class("dim-label");
        toolbar.append(&status_label);

        // Copy button
        let copy_button = Button::with_label(&i18n("Copy"));
        copy_button.set_tooltip_text(Some(&i18n("Copy from remote session to local clipboard")));
        toolbar.append(&copy_button);

        // Paste button
        let paste_button = Button::with_label(&i18n("Paste"));
        paste_button.set_tooltip_text(Some(&i18n("Paste from local clipboard to remote session")));
        toolbar.append(&paste_button);

        // Separator
        let separator = gtk4::Separator::new(Orientation::Vertical);
        separator.set_margin_start(6);
        separator.set_margin_end(6);
        toolbar.append(&separator);

        // Ctrl+Alt+Del button
        let ctrl_alt_del_button = Button::with_label(&i18n("Ctrl+Alt+Del"));
        ctrl_alt_del_button.add_css_class("suggested-action");
        ctrl_alt_del_button.set_tooltip_text(Some(&i18n("Send Ctrl+Alt+Del to remote session")));
        toolbar.append(&ctrl_alt_del_button);

        // Hide toolbar initially
        toolbar.set_visible(false);

        container.append(&toolbar);

        let drawing_area = DrawingArea::new();
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_can_focus(true);
        drawing_area.set_focusable(true);

        container.append(&drawing_area);

        // Reconnect banner (shown when disconnected, at bottom like VTE sessions)
        let reconnect_banner = GtkBox::new(Orientation::Horizontal, 6);
        reconnect_banner.set_margin_start(12);
        reconnect_banner.set_margin_end(12);
        reconnect_banner.set_margin_top(6);
        reconnect_banner.set_margin_bottom(6);
        reconnect_banner.set_halign(gtk4::Align::Center);
        reconnect_banner.set_widget_name("reconnect-banner");
        reconnect_banner.set_visible(false);

        let reconnect_label = Label::new(Some(&i18n("Session disconnected")));
        reconnect_label.add_css_class("dim-label");

        let reconnect_button = Button::with_label(&i18n("Reconnect"));
        reconnect_button.add_css_class("suggested-action");
        reconnect_button.set_tooltip_text(Some(&i18n("Reconnect to this session")));

        reconnect_banner.append(&reconnect_label);
        reconnect_banner.append(&reconnect_button);

        container.append(&reconnect_banner);

        let pixel_buffer = Rc::new(RefCell::new(SpicePixelBuffer::new(1280, 720)));
        let cairo_buffer = Rc::new(RefCell::new(crate::cairo_buffer::CairoBackedBuffer::new(
            1280, 720,
        )));
        let state = Rc::new(RefCell::new(SpiceConnectionState::Disconnected));
        let width = Rc::new(RefCell::new(1280u32));
        let height = Rc::new(RefCell::new(720u32));
        let spice_width = Rc::new(RefCell::new(1280u32));
        let spice_height = Rc::new(RefCell::new(720u32));

        let widget = Self {
            container,
            toolbar,
            status_label,
            copy_button: copy_button.clone(),
            paste_button: paste_button.clone(),
            ctrl_alt_del_button: ctrl_alt_del_button.clone(),
            separator,
            drawing_area,
            pixel_buffer,
            cairo_buffer,
            state,
            config: Rc::new(RefCell::new(None)),
            process: Rc::new(RefCell::new(None)),
            is_embedded: Rc::new(RefCell::new(false)),
            width,
            height,
            spice_width,
            spice_height,
            on_state_changed: Rc::new(RefCell::new(None)),
            on_error: Rc::new(RefCell::new(None)),
            on_reconnect: Rc::new(RefCell::new(None)),
            reconnect_banner,
            reconnect_button,
        };

        widget.setup_drawing();
        widget.setup_resize_handler();
        widget.setup_clipboard_buttons(&copy_button, &paste_button);
        widget.setup_ctrl_alt_del_button(&ctrl_alt_del_button);
        widget.setup_reconnect_button();
        widget.setup_visibility_handler();

        widget
    }

    /// Sets up visibility handler to redraw when widget becomes visible
    fn setup_visibility_handler(&self) {
        let drawing_area = self.drawing_area.clone();
        self.container.connect_map(move |_| {
            drawing_area.queue_draw();
        });
    }

    /// Sets up the reconnect button click handler
    fn setup_reconnect_button(&self) {
        let on_reconnect = self.on_reconnect.clone();

        self.reconnect_button.connect_clicked(move |_| {
            if let Some(ref callback) = *on_reconnect.borrow() {
                callback();
            }
        });
    }

    /// Connects a callback for reconnect button clicks
    ///
    /// The callback is invoked when the user clicks the Reconnect button
    /// after a session has disconnected or encountered an error.
    pub fn connect_reconnect<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.on_reconnect.borrow_mut() = Some(Box::new(callback));
    }

    /// Sets up the drawing function for the DrawingArea
    fn setup_drawing(&self) {
        let pixel_buffer = self.pixel_buffer.clone();
        let cairo_buffer = self.cairo_buffer.clone();
        let state = self.state.clone();
        let is_embedded = self.is_embedded.clone();

        self.drawing_area.set_draw_func(move |_area, cr, w, h| {
            use gtk4::cairo;

            let current_state = *state.borrow();
            let embedded = *is_embedded.borrow();

            // Fill background
            cr.set_source_rgb(0.1, 0.1, 0.1);
            let _ = cr.paint();

            if embedded && current_state == SpiceConnectionState::Connected {
                // Fast path: use the persistent Cairo surface (zero-copy)
                let buffer = cairo_buffer.borrow();
                let buf_w = crate::utils::dimension_to_i32(buffer.width());
                let buf_h = crate::utils::dimension_to_i32(buffer.height());

                let should_render = buf_w > 0 && buf_h > 0 && buffer.has_data();

                if should_render && let Some(surface) = buffer.surface() {
                    let scale_x = f64::from(w) / f64::from(buf_w);
                    let scale_y = f64::from(h) / f64::from(buf_h);
                    let scale = scale_x.min(scale_y);

                    let scaled_w = (f64::from(buf_w) * scale) as i32;
                    let scaled_h = (f64::from(buf_h) * scale) as i32;
                    let offset_x = (w - scaled_w) / 2;
                    let offset_y = (h - scaled_h) / 2;

                    cr.translate(f64::from(offset_x), f64::from(offset_y));
                    cr.scale(scale, scale);
                    let _ = cr.set_source_surface(surface, 0.0, 0.0);
                    let _ = cr.paint();
                    return;
                }

                // Fallback: old SpicePixelBuffer path (to_vec copy)
                #[expect(
    clippy::items_after_statements,
    reason = "local helper introduced inline next to its only call site; hoisting would scatter related logic"
)]
                static WARN_ONCE: std::sync::Once = std::sync::Once::new();
                WARN_ONCE.call_once(|| {
                    tracing::warn!("SPICE: using fallback SpicePixelBuffer with per-frame to_vec() copy — consider migrating to CairoBackedBuffer");
                });
                let fb = pixel_buffer.borrow();
                let fb_w = crate::utils::dimension_to_i32(fb.width());
                let fb_h = crate::utils::dimension_to_i32(fb.height());

                if fb_w > 0 && fb_h > 0 && !fb.data().is_empty() {
                    let scale_x = f64::from(w) / f64::from(fb_w);
                    let scale_y = f64::from(h) / f64::from(fb_h);
                    let scale = scale_x.min(scale_y);

                    let scaled_w = (f64::from(fb_w) * scale) as i32;
                    let scaled_h = (f64::from(fb_h) * scale) as i32;
                    let offset_x = (w - scaled_w) / 2;
                    let offset_y = (h - scaled_h) / 2;

                    let stride = cairo::Format::ARgb32
                        .stride_for_width(fb.width())
                        .unwrap_or(fb_w * 4);

                    if let Ok(surface) = cairo::ImageSurface::create_for_data(
                        fb.data().to_vec(),
                        cairo::Format::ARgb32,
                        fb_w,
                        fb_h,
                        stride,
                    ) {
                        cr.translate(f64::from(offset_x), f64::from(offset_y));
                        cr.scale(scale, scale);
                        let _ = cr.set_source_surface(&surface, 0.0, 0.0);
                        let _ = cr.paint();
                    }
                }
            } else {
                // Show status text
                cr.set_source_rgb(0.7, 0.7, 0.7);
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                cr.set_font_size(13.0);

                let status_text = match current_state {
                    SpiceConnectionState::Disconnected => i18n("Session ended"),
                    SpiceConnectionState::Connecting => i18n("Connecting..."),
                    SpiceConnectionState::Connected if !embedded => {
                        i18n("Session running in external window")
                    }
                    SpiceConnectionState::Connected => i18n("Connected"),
                    SpiceConnectionState::Error => i18n("Connection error"),
                };

                let color = match current_state {
                    SpiceConnectionState::Connected => (0.6, 0.8, 0.6),
                    SpiceConnectionState::Connecting => (0.8, 0.8, 0.6),
                    SpiceConnectionState::Error => (0.8, 0.4, 0.4),
                    SpiceConnectionState::Disconnected => (0.8, 0.4, 0.4),
                };
                cr.set_source_rgb(color.0, color.1, color.2);

                if let Ok(extents) = cr.text_extents(&status_text) {
                    let x = (f64::from(w) - extents.width()) / 2.0;
                    let y = f64::midpoint(f64::from(h), extents.height());
                    cr.move_to(x, y);
                    let _ = cr.show_text(&status_text);
                }
            }
        });
    }

    /// Sets up resize handler
    fn setup_resize_handler(&self) {
        let width = self.width.clone();
        let height = self.height.clone();

        self.drawing_area.connect_resize(move |_, w, h| {
            if w >= 0 && h >= 0 {
                if let Ok(w_u32) = u32::try_from(w) {
                    *width.borrow_mut() = w_u32;
                }
                if let Ok(h_u32) = u32::try_from(h) {
                    *height.borrow_mut() = h_u32;
                }
            }
        });
    }

    /// Sets up clipboard buttons
    fn setup_clipboard_buttons(&self, copy_btn: &Button, paste_btn: &Button) {
        // Clipboard sharing was handled by the removed native SPICE client; the
        // external viewer manages its own clipboard. The buttons stay in the
        // (hidden) toolbar for layout parity but have no handler.
        let _ = copy_btn;
        let _ = paste_btn;
    }

    /// Sets up Ctrl+Alt+Del button
    fn setup_ctrl_alt_del_button(&self, btn: &Button) {
        // Ctrl+Alt+Del forwarding required the removed native input channel;
        // the external viewer provides its own send-key menu.
        let _ = btn;
    }

    /// Returns the main container widget
    #[must_use]
    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Returns the current connection state
    #[must_use]
    pub fn state(&self) -> SpiceConnectionState {
        *self.state.borrow()
    }

    /// Connects a callback for state changes
    pub fn connect_state_changed<F>(&self, callback: F)
    where
        F: Fn(SpiceConnectionState) + 'static,
    {
        let reconnect_banner = self.reconnect_banner.clone();
        let copy_button = self.copy_button.clone();
        let paste_button = self.paste_button.clone();
        let ctrl_alt_del_button = self.ctrl_alt_del_button.clone();
        let separator = self.separator.clone();
        let toolbar = self.toolbar.clone();

        *self.on_state_changed.borrow_mut() = Some(Box::new(move |state| {
            // Update button visibility based on state
            let show_reconnect = matches!(
                state,
                SpiceConnectionState::Disconnected | SpiceConnectionState::Error
            );

            // Show/hide reconnect banner at bottom
            reconnect_banner.set_visible(show_reconnect);

            // When disconnected, hide toolbar buttons
            copy_button.set_visible(!show_reconnect);
            paste_button.set_visible(!show_reconnect);
            ctrl_alt_del_button.set_visible(!show_reconnect);
            separator.set_visible(!show_reconnect);

            // Hide toolbar when disconnected (no reconnect button there anymore)
            if show_reconnect {
                toolbar.set_visible(false);
            }
            // Call the user's callback
            callback(state);
        }));
    }

    /// Connects a callback for errors
    pub fn connect_error<F>(&self, callback: F)
    where
        F: Fn(&str) + 'static,
    {
        *self.on_error.borrow_mut() = Some(Box::new(callback));
    }

    /// Sets the connection state and notifies listeners
    fn set_state(&self, new_state: SpiceConnectionState) {
        *self.state.borrow_mut() = new_state;
        self.drawing_area.queue_draw();

        if let Some(ref callback) = *self.on_state_changed.borrow() {
            callback(new_state);
        }
    }

    /// Reports an error and notifies listeners
    fn report_error(&self, message: &str) {
        self.set_state(SpiceConnectionState::Error);

        if let Some(ref callback) = *self.on_error.borrow() {
            callback(message);
        }
    }

    /// Connects to a SPICE server by launching an external viewer.
    ///
    /// # Errors
    ///
    /// Returns [`SpiceClientError::ConnectionFailed`] if no SPICE viewer is
    /// installed or the viewer process fails to launch.
    pub fn connect(&self, config: &SpiceClientConfig) -> Result<(), SpiceClientError> {
        *self.config.borrow_mut() = Some(config.clone());
        self.set_state(SpiceConnectionState::Connecting);
        self.connect_external(config)
    }

    /// Connects using an external SPICE viewer.
    fn connect_external(&self, config: &SpiceClientConfig) -> Result<(), SpiceClientError> {
        use rustconn_core::spice_client::{SpiceViewerLaunchResult, launch_spice_viewer};

        match launch_spice_viewer(config) {
            SpiceViewerLaunchResult::Launched { viewer, pid } => {
                tracing::info!(%viewer, ?pid, "Launched external SPICE viewer");
                *self.is_embedded.borrow_mut() = false;
                self.set_state(SpiceConnectionState::Connected);
                // Hide toolbar for external mode
                self.toolbar.set_visible(false);
                Ok(())
            }
            SpiceViewerLaunchResult::NoViewerFound => {
                self.report_error("No SPICE viewer found (install remote-viewer or virt-viewer)");
                Err(SpiceClientError::ConnectionFailed(
                    "No SPICE viewer found".to_string(),
                ))
            }
            SpiceViewerLaunchResult::LaunchFailed(msg) => {
                self.report_error(&format!("Failed to launch viewer: {msg}"));
                Err(SpiceClientError::ConnectionFailed(msg))
            }
        }
    }

    /// Disconnects from the SPICE server
    pub fn disconnect(&self) {
        // Kill external process if any
        if let Some(mut process) = self.process.borrow_mut().take() {
            let _ = process.kill();
        }

        *self.is_embedded.borrow_mut() = false;
        self.toolbar.set_visible(false);
        self.set_state(SpiceConnectionState::Disconnected);
    }

    /// Reconnects using the stored configuration
    ///
    /// This method attempts to reconnect to the SPICE server using the
    /// configuration from the previous connection.
    ///
    /// # Errors
    ///
    /// Returns an error if no previous configuration exists or if
    /// the connection fails.
    pub fn reconnect(&self) -> Result<(), SpiceClientError> {
        let config = self.config.borrow().clone();
        if let Some(config) = config {
            self.connect(&config)
        } else {
            Err(SpiceClientError::ConnectionFailed(
                "No previous configuration to reconnect".to_string(),
            ))
        }
    }

    /// Returns whether the widget is connected
    #[must_use]
    pub fn is_connected(&self) -> bool {
        *self.state.borrow() == SpiceConnectionState::Connected
    }

    /// Returns whether using embedded mode
    #[must_use]
    pub fn is_embedded(&self) -> bool {
        *self.is_embedded.borrow()
    }

    /// Returns the current width
    #[must_use]
    pub fn width(&self) -> u32 {
        *self.width.borrow()
    }

    /// Returns the current height
    #[must_use]
    pub fn height(&self) -> u32 {
        *self.height.borrow()
    }
}

impl Default for EmbeddedSpiceWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::embedded_trait::EmbeddedWidget for EmbeddedSpiceWidget {
    fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    fn state(&self) -> crate::embedded_trait::EmbeddedConnectionState {
        match *self.state.borrow() {
            SpiceConnectionState::Disconnected => {
                crate::embedded_trait::EmbeddedConnectionState::Disconnected
            }
            SpiceConnectionState::Connecting => {
                crate::embedded_trait::EmbeddedConnectionState::Connecting
            }
            SpiceConnectionState::Connected => {
                crate::embedded_trait::EmbeddedConnectionState::Connected
            }
            SpiceConnectionState::Error => crate::embedded_trait::EmbeddedConnectionState::Error,
        }
    }

    fn is_embedded(&self) -> bool {
        *self.is_embedded.borrow()
    }

    fn disconnect(&self) -> Result<(), crate::embedded_trait::EmbeddedError> {
        Self::disconnect(self);
        Ok(())
    }

    fn reconnect(&self) -> Result<(), crate::embedded_trait::EmbeddedError> {
        Self::reconnect(self)
            .map_err(|e| crate::embedded_trait::EmbeddedError::ConnectionFailed(e.to_string()))
    }

    fn send_ctrl_alt_del(&self) {
        // No-op: SPICE runs in an external viewer, which handles special keys
        // through its own UI. The native input channel was removed in 0.18.0.
    }

    fn protocol_name(&self) -> &'static str {
        "SPICE"
    }
}

impl Drop for EmbeddedSpiceWidget {
    fn drop(&mut self) {
        Self::disconnect(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spice_connection_state_display() {
        assert_eq!(
            SpiceConnectionState::Disconnected.to_string(),
            "Disconnected"
        );
        assert_eq!(SpiceConnectionState::Connecting.to_string(), "Connecting");
        assert_eq!(SpiceConnectionState::Connected.to_string(), "Connected");
        assert_eq!(SpiceConnectionState::Error.to_string(), "Error");
    }

    #[test]
    fn test_pixel_buffer_new() {
        let buffer = SpicePixelBuffer::new(100, 50);
        assert_eq!(buffer.width(), 100);
        assert_eq!(buffer.height(), 50);
        assert_eq!(buffer.data().len(), 100 * 50 * 4);
    }

    #[test]
    fn test_pixel_buffer_resize() {
        let mut buffer = SpicePixelBuffer::new(100, 50);
        buffer.resize(200, 100);
        assert_eq!(buffer.width(), 200);
        assert_eq!(buffer.height(), 100);
        assert_eq!(buffer.data().len(), 200 * 100 * 4);
    }
}
