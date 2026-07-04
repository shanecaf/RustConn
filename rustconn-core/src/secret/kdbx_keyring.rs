//! System-keyring storage for the KDBX database password.
//!
//! The KDBX (KeePass-compatible) backend reads and writes its `.kdbx` database
//! directly (see [`super::kdbx`]); the database's own unlock password can be
//! cached in the OS keyring (GNOME Keyring / KDE Wallet / macOS Keychain) so the
//! user is not prompted on every launch. These helpers wrap that single entry.

use secrecy::{ExposeSecret, SecretString};

use crate::error::SecretResult;

/// Keyring entry name for the KDBX database password.
const KEY_KDBX_PASSWORD: &str = "kdbx-password";

/// Stores the KDBX database password in the system keyring.
///
/// # Errors
/// Returns `SecretError` if storage fails.
pub async fn store_kdbx_password_in_keyring(password: &SecretString) -> SecretResult<()> {
    super::keyring::store(
        KEY_KDBX_PASSWORD,
        password.expose_secret(),
        "KeePass Database Password",
    )
    .await
}

/// Retrieves the KDBX database password from the system keyring.
///
/// # Errors
/// Returns `SecretError` if retrieval fails.
pub async fn get_kdbx_password_from_keyring() -> SecretResult<Option<SecretString>> {
    super::keyring::lookup(KEY_KDBX_PASSWORD)
        .await
        .map(|opt| opt.map(SecretString::from))
}

/// Deletes the KDBX database password from the system keyring.
///
/// # Errors
/// Returns `SecretError` if deletion fails.
pub async fn delete_kdbx_password_from_keyring() -> SecretResult<()> {
    super::keyring::clear(KEY_KDBX_PASSWORD).await
}
