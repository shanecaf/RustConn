//! Update connection command.

use std::path::Path;

use rustconn_core::config::ConfigManager;

use crate::commands::add::{apply_jump_host_id, parse_auth_method};
use crate::error::CliError;
use crate::util::{create_config_manager, find_connection};

/// Parameters for the `update` command
pub struct UpdateParams<'a> {
    pub name: &'a str,
    pub new_name: Option<&'a str>,
    pub host: Option<&'a str>,
    pub port: Option<u16>,
    pub user: Option<&'a str>,
    pub key: Option<&'a Path>,
    pub auth_method: Option<&'a str>,
    pub device: Option<&'a str>,
    pub baud_rate: Option<u32>,
    pub icon: Option<&'a str>,
    pub ssh_agent_socket: Option<&'a str>,
    pub provider: Option<&'a str>,
    pub hoop_connection_name: Option<&'a str>,
    pub hoop_gateway_url: Option<&'a str>,
    pub hoop_grpc_url: Option<&'a str>,
    pub aws_profile: Option<&'a str>,
    pub aws_region: Option<&'a str>,
    pub gcp_zone: Option<&'a str>,
    pub gcp_project: Option<&'a str>,
    pub resource_group: Option<&'a str>,
    pub bastion_name: Option<&'a str>,
    pub vm_name: Option<&'a str>,
    pub bastion_id: Option<&'a str>,
    pub target_resource_id: Option<&'a str>,
    pub target_private_ip: Option<&'a str>,
    pub teleport_cluster: Option<&'a str>,
    pub boundary_target: Option<&'a str>,
    pub boundary_addr: Option<&'a str>,
    pub custom_command: Option<&'a str>,
    pub jump_host: Option<&'a str>,
    pub keep_alive_interval: Option<u32>,
    pub keep_alive_count: Option<u32>,
    pub ssh_verbose: bool,
    pub ignore_certificate: bool,
    pub tags: Option<&'a str>,
    pub add_tag: &'a [String],
    pub remove_tag: &'a [String],
    pub description: Option<&'a str>,
    pub group: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub window_mode: Option<&'a str>,
    pub skip_port_check: Option<bool>,
}

/// Update connection command handler
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn cmd_update(config_path: Option<&Path>, params: UpdateParams<'_>) -> Result<(), CliError> {
    let config_manager = create_config_manager(config_path)?;

    let mut connections = config_manager
        .load_connections()
        .map_err(|e| CliError::Config(format!("Failed to load connections: {e}")))?;

    let index = {
        let conn = find_connection(&connections, params.name)?;
        connections.iter().position(|c| c.id == conn.id).unwrap()
    };

    // Resolve --jump-host early (before mutable borrow of connection)
    let resolved_jump_id = if let Some(jump_host_ref) = params.jump_host {
        let jump_conn = find_connection(&connections, jump_host_ref)?;
        Some(jump_conn.id)
    } else {
        None
    };

    let connection = &mut connections[index];

    if let Some(new_name) = params.new_name {
        connection.name = new_name.to_string();
    }
    if let Some(host) = params.host {
        connection.host = host.to_string();
    }
    if let Some(port) = params.port {
        connection.port = port;
    }
    if let Some(user) = params.user {
        connection.username = Some(user.to_string());
    }

    // Update SSH-specific fields
    if params.key.is_some() || params.auth_method.is_some() {
        if let rustconn_core::models::ProtocolConfig::Ssh(ref mut cfg) = connection.protocol_config
        {
            if let Some(key_path) = params.key {
                cfg.key_path = Some(key_path.to_path_buf());
            }
            if let Some(method_str) = params.auth_method {
                cfg.auth_method = parse_auth_method(method_str)?;
            }
        } else {
            if params.key.is_some() {
                tracing::warn!("--key is only applicable to SSH connections");
            }
            if params.auth_method.is_some() {
                tracing::warn!("--auth-method is only applicable to SSH connections");
            }
        }
    }

    // Update Serial-specific fields
    if params.device.is_some() || params.baud_rate.is_some() {
        if let rustconn_core::models::ProtocolConfig::Serial(ref mut cfg) =
            connection.protocol_config
        {
            if let Some(dev) = params.device {
                cfg.device = dev.to_string();
            }
            if let Some(baud) = params.baud_rate {
                cfg.baud_rate = crate::util::parse_baud_rate(baud)?;
            }
        } else {
            if params.device.is_some() {
                tracing::warn!("--device is only applicable to Serial connections");
            }
            if params.baud_rate.is_some() {
                tracing::warn!("--baud-rate is only applicable to Serial connections");
            }
        }
    }

    // Update SSH agent socket for SSH/SFTP connections
    if let Some(socket) = params.ssh_agent_socket {
        match connection.protocol_config {
            rustconn_core::models::ProtocolConfig::Ssh(ref mut cfg) => {
                cfg.ssh_agent_socket = Some(socket.to_string());
            }
            rustconn_core::models::ProtocolConfig::Sftp(ref mut cfg) => {
                cfg.ssh_agent_socket = Some(socket.to_string());
            }
            _ => {
                tracing::warn!("--ssh-agent-socket is only applicable to SSH/SFTP connections");
            }
        }
    }

    // Update ZeroTrust provider-specific fields
    if let rustconn_core::models::ProtocolConfig::ZeroTrust(ref mut zt_config) =
        connection.protocol_config
    {
        if let Some(provider) = params.provider {
            tracing::debug!("ZeroTrust provider hint: {provider}");
        }
        match zt_config.provider_config {
            rustconn_core::models::ZeroTrustProviderConfig::HoopDev(ref mut cfg) => {
                if let Some(conn_name) = params.hoop_connection_name {
                    cfg.connection_name = conn_name.to_string();
                }
                if let Some(url) = params.hoop_gateway_url {
                    cfg.gateway_url = Some(url.to_string());
                }
                if let Some(url) = params.hoop_grpc_url {
                    cfg.grpc_url = Some(url.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::AwsSsm(ref mut cfg) => {
                if let Some(profile) = params.aws_profile {
                    cfg.profile = profile.to_string();
                }
                if let Some(region) = params.aws_region {
                    cfg.region = Some(region.to_string());
                }
                if let Some(host) = params.host {
                    cfg.target = host.to_string();
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::GcpIap(ref mut cfg) => {
                if let Some(host) = params.host {
                    cfg.instance = host.to_string();
                }
                if let Some(zone) = params.gcp_zone {
                    cfg.zone = zone.to_string();
                }
                if let Some(project) = params.gcp_project {
                    cfg.project = Some(project.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::AzureBastion(ref mut cfg) => {
                if let Some(host) = params.host {
                    cfg.target_resource_id = host.to_string();
                }
                if let Some(rg) = params.resource_group {
                    cfg.resource_group = rg.to_string();
                }
                if let Some(bn) = params.bastion_name {
                    cfg.bastion_name = bn.to_string();
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::AzureSsh(ref mut cfg) => {
                if let Some(vm) = params.vm_name {
                    cfg.vm_name = vm.to_string();
                }
                if let Some(rg) = params.resource_group {
                    cfg.resource_group = rg.to_string();
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::OciBastion(ref mut cfg) => {
                if let Some(bid) = params.bastion_id {
                    cfg.bastion_id = bid.to_string();
                }
                if let Some(trid) = params.target_resource_id {
                    cfg.target_resource_id = trid.to_string();
                }
                if let Some(tip) = params.target_private_ip {
                    cfg.target_private_ip = tip.to_string();
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::CloudflareAccess(ref mut cfg) => {
                if let Some(host) = params.host {
                    cfg.hostname = host.to_string();
                }
                if let Some(user) = params.user {
                    cfg.username = Some(user.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::Teleport(ref mut cfg) => {
                if let Some(host) = params.host {
                    cfg.host = host.to_string();
                }
                if let Some(user) = params.user {
                    cfg.username = Some(user.to_string());
                }
                if let Some(cluster) = params.teleport_cluster {
                    cfg.cluster = Some(cluster.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::TailscaleSsh(ref mut cfg) => {
                if let Some(host) = params.host {
                    cfg.host = host.to_string();
                }
                if let Some(user) = params.user {
                    cfg.username = Some(user.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::Boundary(ref mut cfg) => {
                if let Some(target) = params.boundary_target {
                    cfg.target = target.to_string();
                }
                if let Some(addr) = params.boundary_addr {
                    cfg.addr = Some(addr.to_string());
                }
            }
            rustconn_core::models::ZeroTrustProviderConfig::Generic(ref mut cfg) => {
                if let Some(cmd) = params.custom_command {
                    cfg.command_template = cmd.to_string();
                }
            }
        }
    }

    connection.updated_at = chrono::Utc::now();

    // Apply SSH keep-alive and verbose settings
    if params.keep_alive_interval.is_some()
        || params.keep_alive_count.is_some()
        || params.ssh_verbose
    {
        match connection.protocol_config {
            rustconn_core::models::ProtocolConfig::Ssh(ref mut cfg) => {
                if let Some(interval) = params.keep_alive_interval {
                    cfg.keep_alive_interval = Some(interval);
                }
                if let Some(count) = params.keep_alive_count {
                    cfg.keep_alive_count_max = Some(count);
                }
                if params.ssh_verbose {
                    cfg.verbose = true;
                }
            }
            rustconn_core::models::ProtocolConfig::Sftp(ref mut cfg) => {
                if let Some(interval) = params.keep_alive_interval {
                    cfg.keep_alive_interval = Some(interval);
                }
                if let Some(count) = params.keep_alive_count {
                    cfg.keep_alive_count_max = Some(count);
                }
                if params.ssh_verbose {
                    cfg.verbose = true;
                }
            }
            _ => {
                if params.keep_alive_interval.is_some() || params.keep_alive_count.is_some() {
                    tracing::warn!(
                        "--keep-alive-interval/--keep-alive-count are only applicable to SSH/SFTP connections"
                    );
                }
                if params.ssh_verbose {
                    tracing::warn!("--ssh-verbose is only applicable to SSH/SFTP connections");
                }
            }
        }
    }

    // Apply RDP ignore-certificate setting
    if params.ignore_certificate {
        if let rustconn_core::models::ProtocolConfig::Rdp(ref mut cfg) = connection.protocol_config
        {
            cfg.ignore_certificate = true;
        } else {
            tracing::warn!("--ignore-certificate is only applicable to RDP connections");
        }
    }

    // Apply pre-resolved jump host ID
    if let Some(jump_id) = resolved_jump_id {
        if jump_id == connection.id {
            return Err(CliError::Config(
                "A connection cannot use itself as a jump host".into(),
            ));
        }
        apply_jump_host_id(connection, jump_id)?;
    }

    if let Some(icon) = params.icon {
        connection.icon = Some(icon.to_string());
    }

    // Apply common metadata: tags, description, domain, window_mode, skip_port_check
    if let Some(tags_str) = params.tags {
        connection.tags = tags_str
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
    }

    for tag in params.add_tag {
        let trimmed = tag.trim();
        if !trimmed.is_empty() && !connection.tags.iter().any(|t| t == trimmed) {
            connection.tags.push(trimmed.to_string());
        }
    }

    if !params.remove_tag.is_empty() {
        connection
            .tags
            .retain(|t| !params.remove_tag.iter().any(|r| r.trim() == t));
    }

    if let Some(desc) = params.description {
        connection.description = if desc.is_empty() {
            None
        } else {
            Some(desc.to_string())
        };
    }

    if let Some(domain) = params.domain {
        connection.domain = if domain.is_empty() {
            None
        } else {
            Some(domain.to_string())
        };
    }

    if let Some(mode_str) = params.window_mode {
        connection.window_mode = match mode_str {
            "external" => rustconn_core::models::WindowMode::External,
            "fullscreen" => rustconn_core::models::WindowMode::Fullscreen,
            _ => rustconn_core::models::WindowMode::Embedded,
        };
    }

    if let Some(flag) = params.skip_port_check {
        connection.skip_port_check = flag;
    }

    // Resolve --group: find or create the group, then assign group_id (defer save)
    let group_to_save = if let Some(group_name) = params.group {
        let mut groups = config_manager
            .load_groups()
            .map_err(|e| CliError::Config(format!("Failed to load groups: {e}")))?;
        let groups_before = groups.len();
        let group_id = crate::util::find_or_create_group_id(&mut groups, group_name)?;
        connection.group_id = Some(group_id);
        if groups.len() > groups_before {
            Some((groups, group_name.to_string()))
        } else {
            None
        }
    } else {
        None
    };

    ConfigManager::validate_connection(connection)
        .map_err(|e| CliError::Config(format!("Invalid connection: {e}")))?;

    let id = connection.id;
    let name = connection.name.clone();

    if let Some((groups, new_group_name)) = group_to_save {
        config_manager
            .save_groups(&groups)
            .map_err(|e| CliError::Config(format!("Failed to save groups: {e}")))?;
        println!("Created group '{new_group_name}'");
    }

    config_manager
        .save_connections(&connections)
        .map_err(|e| CliError::Config(format!("Failed to save connections: {e}")))?;

    println!("Updated connection '{name}' (ID: {id})");

    Ok(())
}
