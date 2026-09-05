//! Delete connection command.

use std::path::Path;

use crate::error::CliError;
use crate::util::{confirm_on_terminal, create_config_manager, find_connection};

/// Delete connection command handler
///
/// # Errors
///
/// Returns:
/// - [`CliError::Config`] when connections cannot be loaded or saved
/// - [`CliError::ConnectionNotFound`] when no connection matches `name`
///
/// In non-interactive mode without `--force` the call returns `Ok(())`
/// without deleting anything (silent abort) to avoid destructive defaults
/// in scripts.
pub(super) fn cmd_delete(
    config_path: Option<&Path>,
    name: &str,
    force: bool,
) -> Result<(), CliError> {
    let config_manager = create_config_manager(config_path)?;

    let connections = config_manager
        .load_connections()
        .map_err(|e| CliError::Config(format!("Failed to load connections: {e}")))?;

    let connection = find_connection(&connections, name)?;
    let id = connection.id;
    let conn_name = connection.name.clone();
    let protocol = format!("{:?}", connection.protocol);

    // A non-interactive stdin is a silent abort here, not an error: this is the
    // documented behaviour of `connection delete` and scripts rely on it. It is
    // also why `confirm_on_terminal` reports three outcomes — `history clear` and
    // `snippet run` make the opposite choice from the same helper.
    if !force
        && !confirm_on_terminal(&format!("Delete connection '{conn_name}' ({protocol})?"))
            .is_confirmed()
    {
        tracing::info!("Delete aborted by user for '{conn_name}'");
        return Ok(());
    }

    let mut connections = connections;
    connections.retain(|c| c.id != id);

    config_manager
        .save_connections(&connections)
        .map_err(|e| CliError::Config(format!("Failed to save connections: {e}")))?;

    println!("Deleted connection '{conn_name}' (ID: {id})");

    Ok(())
}
