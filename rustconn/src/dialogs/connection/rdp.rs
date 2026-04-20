//! RDP protocol options for the connection dialog
//!
//! This module provides the RDP-specific UI components including:
//! - Client mode selection (Embedded/External)
//! - Performance mode (Quality/Balanced/Speed)
//! - Resolution and color depth settings
//! - Audio redirection
//! - RDP Gateway configuration
//! - Shared folders management

// These functions are prepared for future refactoring when dialog.rs is further modularized
#![allow(dead_code)]

use super::protocol_layout::ProtocolLayoutBuilder;
use super::shared_folders;
use super::widgets::{CheckboxRowBuilder, DropdownRowBuilder, EntryRowBuilder};
use adw::prelude::*;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, CheckButton, DropDown, Entry, Label, Orientation, SpinButton};
use libadwaita as adw;
use rustconn_core::models::{RdpClientMode, RdpPerformanceMode, ScaleOverride, SharedFolder};
use std::cell::RefCell;
use std::rc::Rc;

use crate::i18n::i18n;

/// Return type for RDP options creation
#[allow(clippy::type_complexity)]
pub type RdpOptionsWidgets = (
    GtkBox,
    DropDown,                       // client_mode_dropdown
    DropDown,                       // performance_mode_dropdown
    SpinButton,                     // width_spin
    SpinButton,                     // height_spin
    DropDown,                       // color_dropdown
    DropDown,                       // scale_override_dropdown
    CheckButton,                    // audio_check
    Entry,                          // gateway_entry
    SpinButton,                     // gateway_port_spin
    Entry,                          // gateway_username_entry
    CheckButton,                    // disable_nla_check
    CheckButton,                    // clipboard_check
    CheckButton,                    // show_local_cursor_check
    DropDown,                       // jump_host_dropdown
    Rc<RefCell<Vec<SharedFolder>>>, // shared_folders
    gtk4::ListBox,                  // folders_list
    Entry,                          // custom_args_entry
);

/// Creates the RDP options panel using libadwaita components following GNOME HIG.
#[must_use]
pub fn create_rdp_options() -> RdpOptionsWidgets {
    let (container, content) = ProtocolLayoutBuilder::new().build();

    // === Display Group ===
    let (
        display_group,
        client_mode_dropdown,
        performance_mode_dropdown,
        width_spin,
        height_spin,
        color_dropdown,
        scale_override_dropdown,
    ) = create_display_group();
    content.append(&display_group);

    // === Features Group ===
    let (
        features_group,
        audio_check,
        gateway_entry,
        gateway_port_spin,
        gateway_username_entry,
        disable_nla_check,
        clipboard_check,
        show_local_cursor_check,
    ) = create_features_group();
    content.append(&features_group);

    // === Connection Group (Jump Host) ===
    let (connection_group, jump_host_dropdown) = create_connection_group();
    content.append(&connection_group);

    // === Shared Folders Group ===
    let (folders_group, shared_folders, folders_list) =
        shared_folders::create_shared_folders_group();
    content.append(&folders_group);

    // === Advanced Group ===
    let (advanced_group, args_entry) = create_advanced_group();
    content.append(&advanced_group);

    (
        container,
        client_mode_dropdown,
        performance_mode_dropdown,
        width_spin,
        height_spin,
        color_dropdown,
        scale_override_dropdown,
        audio_check,
        gateway_entry,
        gateway_port_spin,
        gateway_username_entry,
        disable_nla_check,
        clipboard_check,
        show_local_cursor_check,
        jump_host_dropdown,
        shared_folders,
        folders_list,
        args_entry,
    )
}

/// Creates the Display preferences group
#[allow(clippy::type_complexity)]
fn create_display_group() -> (
    adw::PreferencesGroup,
    DropDown,
    DropDown,
    SpinButton,
    SpinButton,
    DropDown,
    DropDown,
) {
    let display_group = adw::PreferencesGroup::builder()
        .title(i18n("Display"))
        .build();

    // Client mode dropdown
    let client_mode_items: Vec<String> = vec![
        i18n(RdpClientMode::Embedded.display_name()),
        i18n(RdpClientMode::External.display_name()),
    ];
    let client_mode_strs: Vec<&str> = client_mode_items.iter().map(String::as_str).collect();
    let (client_mode_row, client_mode_dropdown) = DropdownRowBuilder::new("Client Mode")
        .subtitle("Embedded renders in tab, External opens separate window")
        .items(&client_mode_strs)
        .build();
    display_group.add(&client_mode_row);

    // Performance mode dropdown
    let perf_items: Vec<String> = vec![
        i18n(RdpPerformanceMode::Quality.display_name()),
        i18n(RdpPerformanceMode::Balanced.display_name()),
        i18n(RdpPerformanceMode::Speed.display_name()),
    ];
    let perf_strs: Vec<&str> = perf_items.iter().map(String::as_str).collect();
    let (perf_row, performance_mode_dropdown) = DropdownRowBuilder::new("Performance Mode")
        .subtitle("Quality/speed tradeoff for image rendering")
        .items(&perf_strs)
        .selected(1) // Default to Balanced
        .build();
    display_group.add(&perf_row);

    // Resolution
    let res_box = GtkBox::new(Orientation::Horizontal, 4);
    res_box.set_valign(gtk4::Align::Center);
    let width_adj = gtk4::Adjustment::new(1920.0, 640.0, 7680.0, 1.0, 100.0, 0.0);
    let width_spin = SpinButton::builder()
        .adjustment(&width_adj)
        .climb_rate(1.0)
        .digits(0)
        .build();
    let x_label = Label::new(Some("×"));
    let height_adj = gtk4::Adjustment::new(1080.0, 480.0, 4320.0, 1.0, 100.0, 0.0);
    let height_spin = SpinButton::builder()
        .adjustment(&height_adj)
        .climb_rate(1.0)
        .digits(0)
        .build();
    res_box.append(&width_spin);
    res_box.append(&x_label);
    res_box.append(&height_spin);

    let resolution_row = adw::ActionRow::builder()
        .title(i18n("Resolution"))
        .subtitle(i18n("Width × Height in pixels"))
        .build();
    resolution_row.add_suffix(&res_box);
    display_group.add(&resolution_row);

    // Color depth
    let color_items: Vec<String> = vec![
        i18n("32-bit (True Color)"),
        i18n("24-bit"),
        i18n("16-bit (High Color)"),
        i18n("15-bit"),
        i18n("8-bit"),
    ];
    let color_strs: Vec<&str> = color_items.iter().map(String::as_str).collect();
    let (color_row, color_dropdown) = DropdownRowBuilder::new("Color Depth")
        .subtitle("Higher values provide better quality")
        .items(&color_strs)
        .build();
    display_group.add(&color_row);

    // Scale override dropdown (for embedded mode)
    let scale_items: Vec<String> = ScaleOverride::all()
        .iter()
        .map(|s| i18n(s.display_name()))
        .collect();
    let scale_strs: Vec<&str> = scale_items.iter().map(String::as_str).collect();
    let (scale_row, scale_override_dropdown) = DropdownRowBuilder::new("Display Scale")
        .subtitle("Override HiDPI scaling for embedded viewer")
        .items(&scale_strs)
        .build();
    display_group.add(&scale_row);

    // Connect client mode dropdown to show/hide resolution/color rows
    let resolution_row_clone = resolution_row.clone();
    let color_row_clone = color_row.clone();
    let scale_row_clone = scale_row.clone();
    client_mode_dropdown.connect_selected_notify(move |dropdown| {
        let is_embedded = dropdown.selected() == 0;
        resolution_row_clone.set_visible(!is_embedded);
        color_row_clone.set_visible(!is_embedded);
        scale_row_clone.set_visible(is_embedded);
    });

    // Set initial state (Embedded - hide resolution/color, show scale)
    resolution_row.set_visible(false);
    color_row.set_visible(false);
    scale_row.set_visible(true);

    (
        display_group,
        client_mode_dropdown,
        performance_mode_dropdown,
        width_spin,
        height_spin,
        color_dropdown,
        scale_override_dropdown,
    )
}

/// Creates the Features preferences group
#[allow(clippy::type_complexity)]
fn create_features_group() -> (
    adw::PreferencesGroup,
    CheckButton,
    Entry,
    SpinButton,
    Entry,
    CheckButton,
    CheckButton,
    CheckButton,
) {
    let features_group = adw::PreferencesGroup::builder()
        .title(i18n("Features"))
        .build();

    // Audio redirect
    let (audio_row, audio_check) = CheckboxRowBuilder::new("Audio Redirection")
        .subtitle("Play remote audio locally")
        .build();
    features_group.add(&audio_row);

    // Clipboard sharing
    let (clipboard_row, clipboard_check) = CheckboxRowBuilder::new("Clipboard Sharing")
        .subtitle("Synchronize clipboard with remote")
        .active(true)
        .build();
    features_group.add(&clipboard_row);

    // Disable NLA
    let (nla_row, disable_nla_check) = CheckboxRowBuilder::new("Disable NLA")
        .subtitle("Skip Network Level Authentication (less secure)")
        .build();
    features_group.add(&nla_row);

    // Show local cursor
    let (show_cursor_row, show_local_cursor_check) = CheckboxRowBuilder::new("Show Local Cursor")
        .subtitle("Hide to avoid double cursor in embedded mode")
        .active(true)
        .build();
    features_group.add(&show_cursor_row);

    // Gateway
    let (gateway_row, gateway_entry) = EntryRowBuilder::new("RDP Gateway")
        .subtitle("Remote Desktop Gateway server")
        .placeholder("gateway.example.com")
        .build();
    features_group.add(&gateway_row);

    // Gateway port
    let gw_port_adj = gtk4::Adjustment::new(443.0, 1.0, 65535.0, 1.0, 10.0, 0.0);
    let gateway_port_spin = SpinButton::builder()
        .adjustment(&gw_port_adj)
        .climb_rate(1.0)
        .digits(0)
        .valign(gtk4::Align::Center)
        .build();
    let gw_port_row = adw::ActionRow::builder()
        .title(i18n("Gateway Port"))
        .subtitle(i18n("Default: 443"))
        .build();
    gw_port_row.add_suffix(&gateway_port_spin);
    features_group.add(&gw_port_row);

    // Gateway username
    let gateway_username_entry = Entry::builder()
        .hexpand(true)
        .valign(gtk4::Align::Center)
        .placeholder_text(i18n("Same as connection username"))
        .build();
    let gw_user_row = adw::ActionRow::builder()
        .title(i18n("Gateway Username"))
        .subtitle(i18n("If different from connection username"))
        .build();
    gw_user_row.add_suffix(&gateway_username_entry);
    features_group.add(&gw_user_row);

    // Show/hide gateway details based on gateway hostname
    let gw_port_row_clone = gw_port_row.clone();
    let gw_user_row_clone = gw_user_row.clone();
    let has_gateway = !gateway_entry.text().is_empty();
    gw_port_row.set_visible(has_gateway);
    gw_user_row.set_visible(has_gateway);

    gateway_entry.connect_changed(move |entry| {
        let visible = !entry.text().is_empty();
        gw_port_row_clone.set_visible(visible);
        gw_user_row_clone.set_visible(visible);
    });

    (
        features_group,
        audio_check,
        gateway_entry,
        gateway_port_spin,
        gateway_username_entry,
        disable_nla_check,
        clipboard_check,
        show_local_cursor_check,
    )
}

/// Creates the Connection preferences group with Jump Host dropdown
fn create_connection_group() -> (adw::PreferencesGroup, DropDown) {
    let connection_group = adw::PreferencesGroup::builder()
        .title(i18n("Connection"))
        .build();

    // Jump Host dropdown
    let none_items: Vec<String> = vec![i18n("(None)")];
    let none_refs: Vec<&str> = none_items.iter().map(String::as_str).collect();
    let jump_host_list = gtk4::StringList::new(&none_refs);
    let jump_host_dropdown = DropDown::new(Some(jump_host_list), gtk4::Expression::NONE);
    jump_host_dropdown.set_selected(0);
    jump_host_dropdown.set_enable_search(true);
    // Limit width so long hostnames don't stretch the dialog
    jump_host_dropdown.set_size_request(200, -1);
    jump_host_dropdown.set_hexpand(false);

    let jump_host_row = adw::ActionRow::builder()
        .title(i18n("Jump Host"))
        .subtitle(i18n("Tunnel RDP through an SSH connection"))
        .build();
    jump_host_row.add_suffix(&jump_host_dropdown);
    connection_group.add(&jump_host_row);

    (connection_group, jump_host_dropdown)
}

/// Creates the Advanced preferences group
fn create_advanced_group() -> (adw::PreferencesGroup, Entry) {
    let advanced_group = adw::PreferencesGroup::builder()
        .title(i18n("Advanced"))
        .build();

    let (args_row, args_entry) = EntryRowBuilder::new("Custom Arguments")
        .subtitle("Extra FreeRDP command-line options")
        .placeholder("Additional command-line arguments")
        .build();
    advanced_group.add(&args_row);

    (advanced_group, args_entry)
}
