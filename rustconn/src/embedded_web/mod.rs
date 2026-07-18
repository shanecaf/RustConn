//! Embedded web browser widget using WebKitGTK 6.0
//!
//! This module provides the `EmbeddedWebWidget` struct that embeds a WebKitGTK
//! WebView inside a RustConn tab for Web protocol connections. It implements the
//! `EmbeddedWidget` trait for polymorphic handling alongside RDP and VNC sessions.
//!
//! The widget manages:
//! - Per-connection persistent `NetworkSession` (cookies, local storage)
//! - State machine: Disconnected → Connecting → Connected / Error
//! - 60-second load timeout
//! - Navigation toolbar (back/forward/reload/home/zoom)
//! - Credential autofill via JavaScript injection and HTTP Basic Auth
//!
//! # Feature Gate
//!
//! This entire module is gated behind `#[cfg(feature = "web-embedded")]`.

mod autofill;
mod navigation;
mod settings;

pub use autofill::AutofillManager;
pub use navigation::NavigationToolbar;
pub use settings::apply_settings;

use crate::embedded_trait::{
    EmbeddedConnectionState, EmbeddedError, EmbeddedWidget, ErrorCallback, ReconnectCallback,
    StateCallback,
};
use gtk4::glib;
use gtk4::prelude::*;
use secrecy::SecretString;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use webkit6::prelude::*;

use rustconn_core::models::WebConfig;

/// Embedded web browser widget using WebKitGTK 6.0.
///
/// Implements `EmbeddedWidget` for polymorphic handling alongside
/// RDP and VNC sessions in the terminal notebook.
pub struct EmbeddedWebWidget {
    /// Vertical container: toolbar + webview
    container: gtk4::Box,
    /// Navigation toolbar (back/forward/reload/home/zoom)
    toolbar: NavigationToolbar,
    /// WebKitGTK WebView instance
    web_view: webkit6::WebView,
    /// Per-connection network session (persistent cookies/storage).
    /// The field is read by `connect_download_signal` (download manager)
    /// and kept alive for the GTK widget lifecycle so cookies persist.
    #[expect(
        dead_code,
        reason = "kept alive for NetworkSession lifetime and download signal"
    )]
    network_session: webkit6::NetworkSession,
    /// Current connection state
    state: Rc<RefCell<EmbeddedConnectionState>>,
    /// Original configured URL (for Home button / reconnect)
    home_url: Rc<RefCell<String>>,
    /// Connection UUID (for storage path resolution)
    connection_uuid: Uuid,
    /// State change callback
    on_state_changed: Rc<RefCell<Option<StateCallback>>>,
    /// Error callback
    on_error: Rc<RefCell<Option<ErrorCallback>>>,
    /// Reconnect callback
    on_reconnect: Rc<RefCell<Option<ReconnectCallback>>>,
    /// Load timeout source ID
    load_timeout: Rc<RefCell<Option<glib::SourceId>>>,
    /// Autofill manager
    autofill: AutofillManager,
    /// Reconnect banner (shown on disconnect/error).
    /// Appended to the container widget tree; visibility toggled on state changes.
    #[expect(
        dead_code,
        reason = "kept alive in widget tree; visibility toggled by state changes"
    )]
    reconnect_banner: gtk4::Box,
}

/// Validates that a URL has a supported scheme for the embedded web browser.
///
/// Accepts `http://`, `https://`, and `file://` schemes.
/// Rejects empty strings and unsupported schemes without initiating a network request.
///
/// # Errors
///
/// Returns `EmbeddedError::ConfigurationError` if the URL is empty or has
/// an unsupported scheme.
pub fn validate_url(url: &str) -> Result<(), EmbeddedError> {
    if url.is_empty() {
        return Err(EmbeddedError::ConfigurationError(
            "URL is empty".to_string(),
        ));
    }

    let lower = url.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("file://")
    {
        Ok(())
    } else {
        Err(EmbeddedError::ConfigurationError(format!(
            "Unsupported URL scheme: URL must start with http://, https://, or file:// (got: {})",
            url.chars().take(50).collect::<String>()
        )))
    }
}

/// Creates a persistent `NetworkSession` for the given connection UUID.
///
/// Data is stored at `~/.local/share/rustconn/webkit/<uuid>/` and cache at
/// `~/.cache/rustconn/webkit/<uuid>/`. Falls back to an ephemeral session
/// if the directories cannot be created.
pub fn create_network_session(uuid: &Uuid) -> webkit6::NetworkSession {
    let uuid_str = uuid.to_string();

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".local/share"))
        .join("rustconn")
        .join("webkit")
        .join(&uuid_str);

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
        .join("rustconn")
        .join("webkit")
        .join(&uuid_str);

    // Attempt to create directories
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        tracing::warn!(
            path = %data_dir.display(),
            error = %e,
            "Failed to create webkit data directory, using ephemeral session"
        );
        return webkit6::NetworkSession::new_ephemeral();
    }

    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        tracing::warn!(
            path = %cache_dir.display(),
            error = %e,
            "Failed to create webkit cache directory, using ephemeral session"
        );
        return webkit6::NetworkSession::new_ephemeral();
    }

    let data_str = data_dir.to_string_lossy().to_string();
    let cache_str = cache_dir.to_string_lossy().to_string();

    webkit6::NetworkSession::new(Some(&data_str), Some(&cache_str))
}

/// Returns the data directory path for a connection's webkit session.
///
/// Useful for cleanup when a connection is deleted.
#[must_use]
pub fn session_data_dir(uuid: &Uuid) -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".local/share"))
        .join("rustconn")
        .join("webkit")
        .join(uuid.to_string())
}

/// Returns the cache directory path for a connection's webkit session.
///
/// Useful for cleanup when a connection is deleted.
#[must_use]
pub fn session_cache_dir(uuid: &Uuid) -> std::path::PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".cache"))
        .join("rustconn")
        .join("webkit")
        .join(uuid.to_string())
}

impl EmbeddedWebWidget {
    /// Creates a new embedded web widget.
    ///
    /// Validates the URL, creates a persistent network session, configures
    /// WebView settings, connects signals, and begins loading the page.
    ///
    /// # Errors
    ///
    /// Returns `EmbeddedError::ConfigurationError` if the URL is invalid.
    pub fn new(
        connection_uuid: Uuid,
        url: &str,
        config: &WebConfig,
        credentials: Option<(String, SecretString)>,
    ) -> Result<Self, EmbeddedError> {
        // Validate URL before proceeding
        validate_url(url)?;

        // Create persistent network session for this connection
        let network_session = create_network_session(&connection_uuid);

        // Create WebView with the network session
        let web_view = webkit6::WebView::builder()
            .network_session(&network_session)
            .build();

        // Apply settings (JS, user-agent, hardened defaults)
        settings::apply_settings(&web_view, config);

        // Apply persisted zoom level
        if (config.zoom_level - 1.0).abs() > 0.01 {
            web_view.set_zoom_level(config.zoom_level);
        }

        // Create autofill manager
        let autofill = AutofillManager::new(credentials);

        // Create navigation toolbar (placeholder — fully implemented in task 4.1)
        let toolbar = NavigationToolbar::new();

        // Create reconnect banner (hidden by default)
        let reconnect_banner = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        reconnect_banner.set_visible(false);
        reconnect_banner.add_css_class("toolbar");

        // Build vertical container: toolbar + webview
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        container.append(toolbar.widget());
        container.append(&reconnect_banner);
        container.append(&web_view);

        // Make web_view expand to fill available space
        web_view.set_vexpand(true);
        web_view.set_hexpand(true);

        let state = Rc::new(RefCell::new(EmbeddedConnectionState::Disconnected));
        let home_url = Rc::new(RefCell::new(url.to_string()));
        let on_state_changed: Rc<RefCell<Option<StateCallback>>> = Rc::new(RefCell::new(None));
        let on_error: Rc<RefCell<Option<ErrorCallback>>> = Rc::new(RefCell::new(None));
        let on_reconnect: Rc<RefCell<Option<ReconnectCallback>>> = Rc::new(RefCell::new(None));
        let load_timeout: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

        let widget = Self {
            container,
            toolbar,
            web_view,
            network_session,
            state,
            home_url,
            connection_uuid,
            on_state_changed,
            on_error,
            on_reconnect,
            load_timeout,
            autofill,
            reconnect_banner,
        };

        // Bind navigation toolbar to the web view
        widget
            .toolbar
            .bind_to_webview(&widget.web_view, &widget.home_url);

        // Disable autofill button if no credentials available
        if !widget.autofill.is_available() {
            widget.toolbar.autofill_button().set_sensitive(false);
            widget
                .toolbar
                .autofill_button()
                .set_tooltip_text(Some(&crate::i18n::i18n(
                    "No credentials configured for this connection",
                )));
        }

        // Connect load-changed signal
        widget.connect_load_changed_signal();

        // Connect authenticate signal for HTTP Basic/Digest Auth
        widget.connect_authenticate_signal();

        // Connect download signal for file downloads
        widget.connect_download_signal();

        // Set initial state and begin loading
        widget.set_state(EmbeddedConnectionState::Connecting);
        widget.start_load_timeout();
        widget.web_view.load_uri(url);

        Ok(widget)
    }

    /// Sets the connection state and notifies the callback.
    fn set_state(&self, new_state: EmbeddedConnectionState) {
        *self.state.borrow_mut() = new_state;
        if let Some(ref callback) = *self.on_state_changed.borrow() {
            callback(new_state);
        }
    }

    /// Reports an error and transitions to Error state.
    #[expect(
        dead_code,
        reason = "called from error display paths added in future improvements"
    )]
    fn report_error(&self, error: &EmbeddedError) {
        self.set_state(EmbeddedConnectionState::Error);
        if let Some(ref callback) = *self.on_error.borrow() {
            callback(error);
        }
    }

    /// Starts a 60-second load timeout timer.
    ///
    /// If the page does not finish loading within 60 seconds, the widget
    /// transitions to Error state with a timeout indication.
    fn start_load_timeout(&self) {
        self.cancel_load_timeout();

        let state = Rc::clone(&self.state);
        let on_state_changed = Rc::clone(&self.on_state_changed);
        let on_error = Rc::clone(&self.on_error);

        // 60-second timeout for page load
        let source_id = glib::timeout_add_seconds_local_once(60, move || {
            // Only trigger timeout if still in Connecting state
            if *state.borrow() == EmbeddedConnectionState::Connecting {
                *state.borrow_mut() = EmbeddedConnectionState::Error;

                if let Some(ref callback) = *on_state_changed.borrow() {
                    callback(EmbeddedConnectionState::Error);
                }

                let error = EmbeddedError::ConnectionFailed(
                    "Page load timed out after 60 seconds".to_string(),
                );
                if let Some(ref callback) = *on_error.borrow() {
                    callback(&error);
                }
            }
        });

        *self.load_timeout.borrow_mut() = Some(source_id);
    }

    /// Cancels any running load timeout timer.
    fn cancel_load_timeout(&self) {
        if let Some(source_id) = self.load_timeout.borrow_mut().take() {
            source_id.remove();
        }
    }

    /// Connects the WebView `load-changed` signal to update connection state.
    ///
    /// Maps WebKitGTK `LoadEvent` variants to `EmbeddedConnectionState`:
    /// - `Started` / `Redirected` → `Connecting`
    /// - `Finished` → `Connected`
    fn connect_load_changed_signal(&self) {
        let state = Rc::clone(&self.state);
        let on_state_changed = Rc::clone(&self.on_state_changed);
        let load_timeout = Rc::clone(&self.load_timeout);

        self.web_view.connect_load_changed(move |_web_view, event| {
            use webkit6::LoadEvent;

            let new_state = match event {
                LoadEvent::Started | LoadEvent::Redirected => {
                    Some(EmbeddedConnectionState::Connecting)
                }
                LoadEvent::Committed => None, // Intermediate, no state change
                LoadEvent::Finished => {
                    // Cancel timeout on successful load
                    if let Some(source_id) = load_timeout.borrow_mut().take() {
                        source_id.remove();
                    }
                    Some(EmbeddedConnectionState::Connected)
                }
                _ => None,
            };

            if let Some(new_state) = new_state {
                *state.borrow_mut() = new_state;
                if let Some(ref callback) = *on_state_changed.borrow() {
                    callback(new_state);
                }
            }
        });

        // Connect load-failed signal for error handling
        let state_err = Rc::clone(&self.state);
        let on_state_changed_err = Rc::clone(&self.on_state_changed);
        let on_error = Rc::clone(&self.on_error);
        let load_timeout_err = Rc::clone(&self.load_timeout);

        self.web_view
            .connect_load_failed(move |_web_view, _event, _uri, error| {
                // Cancel timeout on failure
                if let Some(source_id) = load_timeout_err.borrow_mut().take() {
                    source_id.remove();
                }

                *state_err.borrow_mut() = EmbeddedConnectionState::Error;
                if let Some(ref callback) = *on_state_changed_err.borrow() {
                    callback(EmbeddedConnectionState::Error);
                }

                // Truncate error description to 200 characters
                let description = error.message().to_string();
                let truncated = if description.len() > 200 {
                    format!("{}...", &description[..197])
                } else {
                    description
                };

                let embedded_error = EmbeddedError::ConnectionFailed(truncated);
                if let Some(ref callback) = *on_error.borrow() {
                    callback(&embedded_error);
                }

                // Return true to indicate we handled the error
                true
            });
    }

    /// Connects the `authenticate` signal for HTTP Basic/Digest auth handling.
    ///
    /// When the WebView encounters an HTTP Basic/Digest authentication challenge,
    /// the stored credentials are used to respond automatically. If no credentials
    /// are configured, the signal is not connected and WebKitGTK shows its
    /// default authentication dialog.
    fn connect_authenticate_signal(&self) {
        if !self.autofill.is_available() {
            return;
        }

        // Clone credentials for the signal closure. The closure needs owned
        // copies since it outlives this function call.
        let username = self.autofill.username().map(String::from);
        let password = self.autofill.password().cloned();

        self.web_view
            .connect_authenticate(move |_web_view, request| {
                use secrecy::ExposeSecret;
                use zeroize::Zeroizing;

                let (Some(user), Some(pwd)) = (&username, &password) else {
                    return false;
                };

                // Wrap the exposed password in Zeroizing to ensure cleanup.
                let password_plain = Zeroizing::new(pwd.expose_secret().to_string());

                // Create a WebKitGTK Credential and authenticate the request.
                // ForSession persistence — valid for the network session lifetime.
                let credential = webkit6::Credential::new(
                    user,
                    &password_plain,
                    webkit6::CredentialPersistence::ForSession,
                );

                request.authenticate(Some(&credential));

                tracing::debug!(
                    host = ?request.host(),
                    scheme = ?request.scheme(),
                    "HTTP authenticate handled with stored credentials"
                );

                // password_plain is zeroized on drop here.
                true
            });
    }

    /// Connects the WebView download signal to handle file downloads.
    ///
    /// When a page initiates a download, the file is saved to the user's
    /// Downloads directory (`~/Downloads/`). A tracing message logs the
    /// download destination.
    fn connect_download_signal(&self) {
        let network_session = self.web_view.network_session().unwrap_or_else(|| {
            // Should never happen — we always create with a network session
            webkit6::NetworkSession::new_ephemeral()
        });

        network_session.connect_download_started(|_session, download| {
            // Set download destination to ~/Downloads/<filename>
            let downloads_dir = dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"));

            // Extract filename from the download response or URI
            let filename = download
                .response()
                .and_then(|r| r.suggested_filename().map(|f| f.to_string()))
                .unwrap_or_else(|| {
                    download
                        .request()
                        .and_then(|req| req.uri().map(|u| u.to_string()))
                        .and_then(|uri| {
                            uri.rsplit('/')
                                .next()
                                .and_then(|seg| seg.split('?').next())
                                .map(String::from)
                        })
                        .unwrap_or_else(|| "download".to_string())
                });

            let dest_path = downloads_dir.join(&filename);
            let dest_uri = format!("file://{}", dest_path.display());
            download.set_destination(&dest_uri);

            tracing::info!(
                destination = %dest_path.display(),
                "Download started"
            );
        });
    }

    /// Connects a state change callback.
    pub fn connect_state_changed<F>(&self, callback: F)
    where
        F: Fn(EmbeddedConnectionState) + 'static,
    {
        *self.on_state_changed.borrow_mut() = Some(Box::new(callback));
    }

    /// Connects an error callback.
    pub fn connect_error<F>(&self, callback: F)
    where
        F: Fn(&EmbeddedError) + 'static,
    {
        *self.on_error.borrow_mut() = Some(Box::new(callback));
    }

    /// Connects a reconnect callback.
    pub fn connect_reconnect<F>(&self, callback: F)
    where
        F: Fn() + 'static,
    {
        *self.on_reconnect.borrow_mut() = Some(Box::new(callback));
    }

    /// Returns the main container widget.
    #[must_use]
    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    /// Disconnects the embedded web session.
    ///
    /// Cancels the load timeout, stops loading, navigates to `about:blank`,
    /// and transitions to Disconnected state.
    ///
    /// # Errors
    ///
    /// Currently infallible but returns `Result` for `EmbeddedWidget` trait
    /// contract compatibility.
    pub fn disconnect(&self) -> Result<(), EmbeddedError> {
        self.cancel_load_timeout();
        self.web_view.stop_loading();
        self.web_view.load_uri("about:blank");
        self.set_state(EmbeddedConnectionState::Disconnected);
        Ok(())
    }

    /// Reconnects by reloading the original (home) URL.
    ///
    /// # Errors
    ///
    /// Returns `EmbeddedError::ConfigurationError` if the home URL is no
    /// longer valid.
    pub fn reconnect(&self) -> Result<(), EmbeddedError> {
        let url = self.home_url.borrow().clone();
        validate_url(&url)?;
        self.set_state(EmbeddedConnectionState::Connecting);
        self.start_load_timeout();
        self.web_view.load_uri(&url);
        Ok(())
    }

    /// Returns a reference to the WebView.
    #[must_use]
    pub fn web_view(&self) -> &webkit6::WebView {
        &self.web_view
    }

    /// Returns a reference to the navigation toolbar.
    #[must_use]
    pub fn toolbar(&self) -> &NavigationToolbar {
        &self.toolbar
    }

    /// Returns a reference to the autofill manager.
    #[must_use]
    pub fn autofill_manager(&self) -> &AutofillManager {
        &self.autofill
    }

    /// Returns the connection UUID.
    #[must_use]
    pub fn connection_uuid(&self) -> Uuid {
        self.connection_uuid
    }
}

impl EmbeddedWidget for EmbeddedWebWidget {
    fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    fn state(&self) -> EmbeddedConnectionState {
        *self.state.borrow()
    }

    fn is_embedded(&self) -> bool {
        true
    }

    fn disconnect(&self) -> Result<(), EmbeddedError> {
        self.cancel_load_timeout();
        self.web_view.stop_loading();
        self.web_view.load_uri("about:blank");
        self.set_state(EmbeddedConnectionState::Disconnected);
        Ok(())
    }

    fn reconnect(&self) -> Result<(), EmbeddedError> {
        let url = self.home_url.borrow().clone();
        validate_url(&url)?;
        self.set_state(EmbeddedConnectionState::Connecting);
        self.start_load_timeout();
        self.web_view.load_uri(&url);
        Ok(())
    }

    fn send_ctrl_alt_del(&self) {
        // No-op: Web protocol does not support remote key injection.
    }

    fn protocol_name(&self) -> &'static str {
        "Web"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_accepts_http() {
        assert!(validate_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_accepts_https() {
        assert!(validate_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_accepts_file() {
        assert!(validate_url("file:///path/to/file.html").is_ok());
    }

    #[test]
    fn test_validate_url_rejects_empty() {
        let result = validate_url("");
        assert!(result.is_err());
        match result.unwrap_err() {
            EmbeddedError::ConfigurationError(msg) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_validate_url_rejects_unsupported_scheme() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("ssh://example.com").is_err());
        assert!(validate_url("example.com").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_case_insensitive() {
        assert!(validate_url("HTTP://example.com").is_ok());
        assert!(validate_url("HTTPS://example.com").is_ok());
        assert!(validate_url("FILE:///path").is_ok());
    }

    #[test]
    fn test_session_data_dir_uses_uuid() {
        let uuid = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let path = session_data_dir(&uuid);
        assert!(
            path.to_string_lossy()
                .contains("12345678-1234-1234-1234-123456789abc")
        );
        assert!(path.to_string_lossy().contains("webkit"));
    }

    #[test]
    fn test_session_cache_dir_uses_uuid() {
        let uuid = Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap();
        let path = session_cache_dir(&uuid);
        assert!(
            path.to_string_lossy()
                .contains("12345678-1234-1234-1234-123456789abc")
        );
        assert!(path.to_string_lossy().contains("webkit"));
    }

    #[test]
    fn test_session_dirs_are_isolated() {
        let uuid1 = Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        let uuid2 = Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();

        let data1 = session_data_dir(&uuid1);
        let data2 = session_data_dir(&uuid2);
        assert_ne!(data1, data2);

        let cache1 = session_cache_dir(&uuid1);
        let cache2 = session_cache_dir(&uuid2);
        assert_ne!(cache1, cache2);

        // Ensure no path is a prefix of the other
        assert!(!data1.starts_with(&data2));
        assert!(!data2.starts_with(&data1));
    }

    // Feature: embedded-web-browser, Property 1: URL Validation Round-Trip
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        /// Supported URL schemes for the embedded web browser.
        const SUPPORTED_SCHEMES: &[&str] = &["http://", "https://", "file://"];

        /// Strategy generating valid URLs with supported schemes and arbitrary suffixes.
        fn arb_valid_url() -> impl Strategy<Value = String> {
            let scheme = prop_oneof![
                Just("http://".to_string()),
                Just("https://".to_string()),
                Just("file://".to_string()),
                Just("HTTP://".to_string()),
                Just("HTTPS://".to_string()),
                Just("FILE://".to_string()),
                Just("Http://".to_string()),
                Just("Https://".to_string()),
                Just("File://".to_string()),
            ];
            let suffix = "[a-z0-9./]{0,50}";
            (scheme, suffix).prop_map(|(s, rest)| format!("{s}{rest}"))
        }

        /// Strategy generating URLs with random schemes (likely unsupported).
        fn arb_scheme_url() -> impl Strategy<Value = String> {
            let scheme = "[a-z]{0,10}://";
            let suffix = "[a-z0-9./]{0,50}";
            (scheme, suffix).prop_map(|(s, rest)| format!("{s}{rest}"))
        }

        /// Strategy generating completely arbitrary strings (most will be invalid URLs).
        fn arb_arbitrary_string() -> impl Strategy<Value = String> {
            ".*"
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            /// **Feature: embedded-web-browser, Property 1: URL Validation Round-Trip**
            /// **Validates: Requirements 2.2, 2.8**
            ///
            /// For any string accepted by validate_url(), it starts with a supported
            /// scheme (http://, https://, file://) and is non-empty.
            #[test]
            fn accepted_urls_start_with_supported_scheme(url in arb_valid_url()) {
                let result = validate_url(&url);
                // All generated valid URLs should be accepted
                prop_assert!(
                    result.is_ok(),
                    "URL with supported scheme should be accepted: {:?}",
                    url
                );
                // Double-check: accepted URL must be non-empty and start with a supported scheme
                prop_assert!(!url.is_empty(), "Accepted URL must not be empty");
                let lower = url.to_lowercase();
                prop_assert!(
                    SUPPORTED_SCHEMES.iter().any(|s| lower.starts_with(s)),
                    "Accepted URL must start with a supported scheme, got: {:?}",
                    url
                );
            }

            /// **Feature: embedded-web-browser, Property 1: URL Validation Round-Trip**
            /// **Validates: Requirements 2.2, 2.8**
            ///
            /// For any string without a supported scheme or that is empty,
            /// validate_url() returns an error.
            #[test]
            fn urls_without_supported_scheme_are_rejected(url in arb_scheme_url()) {
                let lower = url.to_lowercase();
                let has_supported_scheme = SUPPORTED_SCHEMES.iter().any(|s| lower.starts_with(s));

                if !has_supported_scheme {
                    let result = validate_url(&url);
                    prop_assert!(
                        result.is_err(),
                        "URL without supported scheme should be rejected: {:?}",
                        url
                    );
                }
            }

            /// **Feature: embedded-web-browser, Property 1: URL Validation Round-Trip**
            /// **Validates: Requirements 2.2, 2.8**
            ///
            /// For any arbitrary string, if validate_url() accepts it, then it must
            /// start with a supported scheme and be non-empty. If it does not start
            /// with a supported scheme or is empty, it must be rejected.
            #[test]
            fn validation_round_trip_arbitrary_strings(url in arb_arbitrary_string()) {
                let result = validate_url(&url);
                let lower = url.to_lowercase();
                let has_supported_scheme = SUPPORTED_SCHEMES.iter().any(|s| lower.starts_with(s));

                if url.is_empty() || !has_supported_scheme {
                    prop_assert!(
                        result.is_err(),
                        "URL that is empty or lacks supported scheme should be rejected: {:?}",
                        url
                    );
                } else {
                    prop_assert!(
                        result.is_ok(),
                        "URL with supported scheme should be accepted: {:?}",
                        url
                    );
                }
            }

            // ========== Property 3: Session Isolation ==========

            /// **Feature: embedded-web-browser, Property 3: Session Isolation**
            /// **Validates: Requirements 4.1, 4.6**
            ///
            /// For any two distinct UUIDs, session_data_dir produces non-overlapping
            /// directory paths (neither is a prefix of the other), ensuring cookies and
            /// local storage written by one connection are never readable by the other.
            // Feature: embedded-web-browser, Property 3: Session Isolation
            #[test]
            fn session_data_dirs_are_non_overlapping(
                bytes_a in proptest::array::uniform16(any::<u8>()),
                bytes_b in proptest::array::uniform16(any::<u8>()),
            ) {
                let uuid_a = uuid::Builder::from_bytes(bytes_a)
                    .with_version(uuid::Version::Random)
                    .into_uuid();
                let uuid_b = uuid::Builder::from_bytes(bytes_b)
                    .with_version(uuid::Version::Random)
                    .into_uuid();

                prop_assume!(uuid_a != uuid_b);

                let path_a = session_data_dir(&uuid_a);
                let path_b = session_data_dir(&uuid_b);

                // Paths must be different
                prop_assert_ne!(
                    &path_a, &path_b,
                    "Distinct UUIDs must produce different data directories"
                );

                // Neither path is a prefix of the other (non-overlapping)
                prop_assert!(
                    !path_a.starts_with(&path_b),
                    "Data dir for {:?} should not be a subdirectory of {:?}",
                    uuid_a, uuid_b
                );
                prop_assert!(
                    !path_b.starts_with(&path_a),
                    "Data dir for {:?} should not be a subdirectory of {:?}",
                    uuid_b, uuid_a
                );
            }

            /// **Feature: embedded-web-browser, Property 3: Session Isolation**
            /// **Validates: Requirements 4.1, 4.6**
            ///
            /// For any two distinct UUIDs, session_cache_dir produces non-overlapping
            /// directory paths, ensuring HTTP disk cache written by one connection is
            /// never accessible to the other.
            // Feature: embedded-web-browser, Property 3: Session Isolation
            #[test]
            fn session_cache_dirs_are_non_overlapping(
                bytes_a in proptest::array::uniform16(any::<u8>()),
                bytes_b in proptest::array::uniform16(any::<u8>()),
            ) {
                let uuid_a = uuid::Builder::from_bytes(bytes_a)
                    .with_version(uuid::Version::Random)
                    .into_uuid();
                let uuid_b = uuid::Builder::from_bytes(bytes_b)
                    .with_version(uuid::Version::Random)
                    .into_uuid();

                prop_assume!(uuid_a != uuid_b);

                let path_a = session_cache_dir(&uuid_a);
                let path_b = session_cache_dir(&uuid_b);

                // Paths must be different
                prop_assert_ne!(
                    &path_a, &path_b,
                    "Distinct UUIDs must produce different cache directories"
                );

                // Neither path is a prefix of the other (non-overlapping)
                prop_assert!(
                    !path_a.starts_with(&path_b),
                    "Cache dir for {:?} should not be a subdirectory of {:?}",
                    uuid_a, uuid_b
                );
                prop_assert!(
                    !path_b.starts_with(&path_a),
                    "Cache dir for {:?} should not be a subdirectory of {:?}",
                    uuid_b, uuid_a
                );
            }
        }
    }
}
