//! Property-based tests for `ZeroTrustProviderConfig::from_wizard_fields()`
//!
//! Verifies that all 10 Zero Trust providers correctly map
//! `zt_field1`/`zt_field2`/`zt_field3` into the corresponding config structs.

use proptest::prelude::*;
use rustconn_core::models::{ZeroTrustProvider, ZeroTrustProviderConfig};

// ============================================================================
// Generators
// ============================================================================

/// Arbitrary non-empty string for required fields
fn arb_field() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9._/-]{1,50}"
}

/// Arbitrary optional string field
fn arb_opt_field() -> impl Strategy<Value = Option<String>> {
    prop::option::of(arb_field())
}

// ============================================================================
// Tests: Generic provider
// ============================================================================

proptest! {
    #[test]
    fn generic_maps_command_to_template(command in arb_field()) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::Generic,
            Some(&command),
            None,
            None,
            None,
        );
        match config {
            ZeroTrustProviderConfig::Generic(cfg) => {
                prop_assert_eq!(cfg.command_template, command);
            }
            other => prop_assert!(false, "Expected Generic, got {:?}", other),
        }
    }

    #[test]
    fn generic_none_command_defaults_to_empty(_dummy in 0..1u8) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::Generic,
            None,
            None,
            None,
            None,
        );
        match config {
            ZeroTrustProviderConfig::Generic(cfg) => {
                prop_assert_eq!(cfg.command_template, "");
            }
            other => prop_assert!(false, "Expected Generic, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: AWS SSM
// ============================================================================

proptest! {
    #[test]
    fn aws_ssm_maps_all_fields(
        target in arb_field(),
        region in arb_opt_field(),
        profile in arb_opt_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::AwsSsm,
            None,
            Some(&target),
            region.as_deref(),
            profile.as_deref(),
        );
        match config {
            ZeroTrustProviderConfig::AwsSsm(cfg) => {
                prop_assert_eq!(&cfg.target, &target);
                prop_assert_eq!(&cfg.region, &region);
                let expected_profile = profile.unwrap_or_else(|| "default".to_string());
                prop_assert_eq!(&cfg.profile, &expected_profile);
            }
            other => prop_assert!(false, "Expected AwsSsm, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: GCP IAP
// ============================================================================

proptest! {
    #[test]
    fn gcp_iap_maps_all_fields(
        instance in arb_field(),
        zone in arb_field(),
        project in arb_opt_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::GcpIap,
            None,
            Some(&instance),
            Some(&zone),
            project.as_deref(),
        );
        match config {
            ZeroTrustProviderConfig::GcpIap(cfg) => {
                prop_assert_eq!(&cfg.instance, &instance);
                prop_assert_eq!(&cfg.zone, &zone);
                prop_assert_eq!(&cfg.project, &project);
            }
            other => prop_assert!(false, "Expected GcpIap, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Azure Bastion
// ============================================================================

proptest! {
    #[test]
    fn azure_bastion_maps_all_fields(
        resource_id in arb_field(),
        rg in arb_field(),
        bastion_name in arb_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::AzureBastion,
            None,
            Some(&resource_id),
            Some(&rg),
            Some(&bastion_name),
        );
        match config {
            ZeroTrustProviderConfig::AzureBastion(cfg) => {
                prop_assert_eq!(&cfg.target_resource_id, &resource_id);
                prop_assert_eq!(&cfg.resource_group, &rg);
                prop_assert_eq!(&cfg.bastion_name, &bastion_name);
            }
            other => prop_assert!(false, "Expected AzureBastion, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Azure SSH
// ============================================================================

proptest! {
    #[test]
    fn azure_ssh_maps_all_fields(
        vm_name in arb_field(),
        rg in arb_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::AzureSsh,
            None,
            Some(&vm_name),
            Some(&rg),
            None,
        );
        match config {
            ZeroTrustProviderConfig::AzureSsh(cfg) => {
                prop_assert_eq!(&cfg.vm_name, &vm_name);
                prop_assert_eq!(&cfg.resource_group, &rg);
            }
            other => prop_assert!(false, "Expected AzureSsh, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Cloudflare Access
// ============================================================================

proptest! {
    #[test]
    fn cloudflare_maps_hostname(hostname in arb_field()) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::CloudflareAccess,
            None,
            Some(&hostname),
            None,
            None,
        );
        match config {
            ZeroTrustProviderConfig::CloudflareAccess(cfg) => {
                prop_assert_eq!(&cfg.hostname, &hostname);
                prop_assert_eq!(cfg.username, None);
            }
            other => prop_assert!(false, "Expected CloudflareAccess, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Teleport
// ============================================================================

proptest! {
    #[test]
    fn teleport_maps_all_fields(
        host in arb_field(),
        cluster in arb_opt_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::Teleport,
            None,
            Some(&host),
            cluster.as_deref(),
            None,
        );
        match config {
            ZeroTrustProviderConfig::Teleport(cfg) => {
                prop_assert_eq!(&cfg.host, &host);
                prop_assert_eq!(cfg.username, None);
                prop_assert_eq!(&cfg.cluster, &cluster);
            }
            other => prop_assert!(false, "Expected Teleport, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Tailscale SSH
// ============================================================================

proptest! {
    #[test]
    fn tailscale_maps_host(host in arb_field()) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::TailscaleSsh,
            None,
            Some(&host),
            None,
            None,
        );
        match config {
            ZeroTrustProviderConfig::TailscaleSsh(cfg) => {
                prop_assert_eq!(&cfg.host, &host);
                prop_assert_eq!(cfg.username, None);
            }
            other => prop_assert!(false, "Expected TailscaleSsh, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Boundary
// ============================================================================

proptest! {
    #[test]
    fn boundary_maps_all_fields(
        target in arb_field(),
        addr in arb_opt_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::Boundary,
            None,
            Some(&target),
            addr.as_deref(),
            None,
        );
        match config {
            ZeroTrustProviderConfig::Boundary(cfg) => {
                prop_assert_eq!(&cfg.target, &target);
                prop_assert_eq!(&cfg.addr, &addr);
            }
            other => prop_assert!(false, "Expected Boundary, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: Hoop.dev
// ============================================================================

proptest! {
    #[test]
    fn hoop_dev_maps_all_fields(
        connection_name in arb_field(),
        gateway_url in arb_opt_field(),
    ) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::HoopDev,
            None,
            Some(&connection_name),
            gateway_url.as_deref(),
            None,
        );
        match config {
            ZeroTrustProviderConfig::HoopDev(cfg) => {
                prop_assert_eq!(&cfg.connection_name, &connection_name);
                prop_assert_eq!(&cfg.gateway_url, &gateway_url);
                prop_assert_eq!(cfg.grpc_url, None);
            }
            other => prop_assert!(false, "Expected HoopDev, got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: OCI Bastion (falls back to Generic)
// ============================================================================

proptest! {
    #[test]
    fn oci_bastion_falls_back_to_generic(command in arb_field()) {
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            ZeroTrustProvider::OciBastion,
            Some(&command),
            None,
            None,
            None,
        );
        match config {
            ZeroTrustProviderConfig::Generic(cfg) => {
                prop_assert_eq!(cfg.command_template, command);
            }
            other => prop_assert!(false, "Expected Generic (OCI fallback), got {:?}", other),
        }
    }
}

// ============================================================================
// Tests: None fields default correctly
// ============================================================================

proptest! {
    #[test]
    fn all_none_fields_produce_valid_config(provider_idx in 0u8..11) {
        let provider = match provider_idx {
            0 => ZeroTrustProvider::Generic,
            1 => ZeroTrustProvider::AwsSsm,
            2 => ZeroTrustProvider::GcpIap,
            3 => ZeroTrustProvider::AzureBastion,
            4 => ZeroTrustProvider::AzureSsh,
            5 => ZeroTrustProvider::CloudflareAccess,
            6 => ZeroTrustProvider::Teleport,
            7 => ZeroTrustProvider::TailscaleSsh,
            8 => ZeroTrustProvider::Boundary,
            9 => ZeroTrustProvider::HoopDev,
            _ => ZeroTrustProvider::OciBastion,
        };

        // Should not panic with all None fields
        let config = ZeroTrustProviderConfig::from_wizard_fields(
            provider, None, None, None, None,
        );

        // Verify the variant matches the provider (except OCI → Generic)
        match (provider, &config) {
            (ZeroTrustProvider::Generic, ZeroTrustProviderConfig::Generic(_)) => {}
            (ZeroTrustProvider::AwsSsm, ZeroTrustProviderConfig::AwsSsm(cfg)) => {
                prop_assert_eq!(&cfg.profile, "default");
            }
            (ZeroTrustProvider::GcpIap, ZeroTrustProviderConfig::GcpIap(_)) => {}
            (ZeroTrustProvider::AzureBastion, ZeroTrustProviderConfig::AzureBastion(_)) => {}
            (ZeroTrustProvider::AzureSsh, ZeroTrustProviderConfig::AzureSsh(_)) => {}
            (ZeroTrustProvider::CloudflareAccess, ZeroTrustProviderConfig::CloudflareAccess(_)) => {}
            (ZeroTrustProvider::Teleport, ZeroTrustProviderConfig::Teleport(_)) => {}
            (ZeroTrustProvider::TailscaleSsh, ZeroTrustProviderConfig::TailscaleSsh(_)) => {}
            (ZeroTrustProvider::Boundary, ZeroTrustProviderConfig::Boundary(_)) => {}
            (ZeroTrustProvider::HoopDev, ZeroTrustProviderConfig::HoopDev(_)) => {}
            (ZeroTrustProvider::OciBastion, ZeroTrustProviderConfig::Generic(_)) => {}
            (p, c) => prop_assert!(false, "Mismatched provider {:?} → config {:?}", p, c),
        }
    }
}
