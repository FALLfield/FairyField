//! Tool permissions management
//!
//! Manages permission grants and checks for tool execution.
//! Each tool declares required permissions via ToolManifest, and
//! the permission system gates execution based on user-granted permissions.

use crate::tools::manifest::ToolPermission;

/// Check if a tool is allowed to execute based on its required permissions
/// and the set of currently granted permissions.
pub fn check_permissions(
    _tool_id: &str,
    required: &[ToolPermission],
    granted: &[ToolPermission],
) -> bool {
    // All required permissions must be present in granted set
    required.iter().all(|req| granted.contains(req))
}

/// List all known tool permissions for discovery
pub fn all_permissions() -> Vec<ToolPermission> {
    vec![
        ToolPermission::ReadFiles,
        ToolPermission::WriteFiles,
        ToolPermission::ExecuteCommand,
        ToolPermission::NetworkAccess,
        ToolPermission::FileSystem,
        ToolPermission::UserData,
        ToolPermission::CodeExecution,
        ToolPermission::BrowserControl,
        ToolPermission::Notifications,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grants_when_all_permissions_present() {
        let required = vec![ToolPermission::ReadFiles];
        let granted = vec![ToolPermission::ReadFiles, ToolPermission::WriteFiles];
        assert!(check_permissions("test", &required, &granted));
    }

    #[test]
    fn denies_when_permission_missing() {
        let required = vec![ToolPermission::NetworkAccess];
        let granted = vec![ToolPermission::ReadFiles];
        assert!(!check_permissions("test", &required, &granted));
    }

    #[test]
    fn grants_empty_requirements() {
        let required: Vec<ToolPermission> = vec![];
        let granted: Vec<ToolPermission> = vec![];
        assert!(check_permissions("test", &required, &granted));
    }

    #[test]
    fn all_permissions_list_is_nonempty() {
        let all = all_permissions();
        assert!(!all.is_empty());
    }
}
