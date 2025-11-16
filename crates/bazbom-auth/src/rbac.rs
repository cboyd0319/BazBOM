//! Role-Based Access Control (RBAC)
//!
//! Defines roles and permissions for BazBOM users.
//!
//! # Role Hierarchy
//!
//! - **Admin**: Full access to all resources
//! - **SecurityLead**: Manage vulnerabilities, policies, team assignments
//! - **Developer**: View vulnerabilities, update assignments
//! - **User**: Read-only access to SBOMs and scans
//! - **CI**: Limited access for CI/CD automation

use serde::{Deserialize, Serialize};

/// User roles in BazBOM
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Full administrative access
    Admin,
    /// Security team lead - manage security operations
    SecurityLead,
    /// Developer - view and update vulnerabilities
    Developer,
    /// Read-only user
    User,
    /// CI/CD system - automated scanning
    CI,
}

impl Role {
    /// Get all permissions for this role
    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::Admin => vec![
                Permission::ReadSBOM,
                Permission::WriteSBOM,
                Permission::ReadVulnerabilities,
                Permission::WriteVulnerabilities,
                Permission::ManageUsers,
                Permission::ManageKeys,
                Permission::ManagePolicy,
                Permission::ViewAuditLogs,
                Permission::ManageTeam,
                Permission::ExportData,
            ],
            Role::SecurityLead => vec![
                Permission::ReadSBOM,
                Permission::WriteSBOM,
                Permission::ReadVulnerabilities,
                Permission::WriteVulnerabilities,
                Permission::ManagePolicy,
                Permission::ViewAuditLogs,
                Permission::ManageTeam,
                Permission::ExportData,
            ],
            Role::Developer => vec![
                Permission::ReadSBOM,
                Permission::ReadVulnerabilities,
                Permission::WriteVulnerabilities,
                Permission::ExportData,
            ],
            Role::User => vec![
                Permission::ReadSBOM,
                Permission::ReadVulnerabilities,
            ],
            Role::CI => vec![
                Permission::ReadSBOM,
                Permission::WriteSBOM,
                Permission::ReadVulnerabilities,
            ],
        }
    }

    /// Check if this role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions().contains(permission)
    }

    /// Check if this role includes another role's permissions
    pub fn includes_role(&self, other: &Role) -> bool {
        let self_perms = self.permissions();
        let other_perms = other.permissions();

        other_perms.iter().all(|p| self_perms.contains(p))
    }
}

/// Permissions for fine-grained access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read SBOM data
    ReadSBOM,
    /// Create/update SBOM data
    WriteSBOM,
    /// View vulnerabilities
    ReadVulnerabilities,
    /// Update vulnerability status, assignments
    WriteVulnerabilities,
    /// Manage user accounts
    ManageUsers,
    /// Manage API keys
    ManageKeys,
    /// Manage security policies
    ManagePolicy,
    /// View audit logs
    ViewAuditLogs,
    /// Manage team assignments and metrics
    ManageTeam,
    /// Export data (PDF, CSV, etc.)
    ExportData,
}

/// Authorization checker
pub struct Authorizer;

impl Authorizer {
    /// Check if user has required permission
    pub fn check_permission(roles: &[Role], required: &Permission) -> bool {
        roles.iter().any(|role| role.has_permission(required))
    }

    /// Check if user has ALL required permissions
    pub fn check_all_permissions(roles: &[Role], required: &[Permission]) -> bool {
        required
            .iter()
            .all(|perm| Self::check_permission(roles, perm))
    }

    /// Check if user has ANY of the required permissions
    pub fn check_any_permission(roles: &[Role], required: &[Permission]) -> bool {
        required
            .iter()
            .any(|perm| Self::check_permission(roles, perm))
    }

    /// Check if user has specific role
    pub fn has_role(roles: &[Role], required: &Role) -> bool {
        roles.contains(required)
    }

    /// Check if user is admin
    pub fn is_admin(roles: &[Role]) -> bool {
        Self::has_role(roles, &Role::Admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let admin = Role::Admin;

        assert!(admin.has_permission(&Permission::ReadSBOM));
        assert!(admin.has_permission(&Permission::WriteSBOM));
        assert!(admin.has_permission(&Permission::ManageUsers));
        assert!(admin.has_permission(&Permission::ViewAuditLogs));
    }

    #[test]
    fn test_user_limited_permissions() {
        let user = Role::User;

        assert!(user.has_permission(&Permission::ReadSBOM));
        assert!(user.has_permission(&Permission::ReadVulnerabilities));
        assert!(!user.has_permission(&Permission::WriteSBOM));
        assert!(!user.has_permission(&Permission::ManageUsers));
    }

    #[test]
    fn test_developer_permissions() {
        let dev = Role::Developer;

        assert!(dev.has_permission(&Permission::ReadSBOM));
        assert!(dev.has_permission(&Permission::ReadVulnerabilities));
        assert!(dev.has_permission(&Permission::WriteVulnerabilities));
        assert!(!dev.has_permission(&Permission::ManageUsers));
    }

    #[test]
    fn test_ci_permissions() {
        let ci = Role::CI;

        assert!(ci.has_permission(&Permission::ReadSBOM));
        assert!(ci.has_permission(&Permission::WriteSBOM));
        assert!(ci.has_permission(&Permission::ReadVulnerabilities));
        assert!(!ci.has_permission(&Permission::WriteVulnerabilities));
        assert!(!ci.has_permission(&Permission::ManageUsers));
    }

    #[test]
    fn test_role_hierarchy() {
        let admin = Role::Admin;
        let user = Role::User;

        assert!(admin.includes_role(&user));
        assert!(!user.includes_role(&admin));
    }

    #[test]
    fn test_authorizer_check_permission() {
        let roles = vec![Role::Developer];

        assert!(Authorizer::check_permission(&roles, &Permission::ReadSBOM));
        assert!(Authorizer::check_permission(&roles, &Permission::WriteVulnerabilities));
        assert!(!Authorizer::check_permission(&roles, &Permission::ManageUsers));
    }

    #[test]
    fn test_authorizer_check_all_permissions() {
        let roles = vec![Role::Developer];

        assert!(Authorizer::check_all_permissions(
            &roles,
            &[Permission::ReadSBOM, Permission::ReadVulnerabilities]
        ));

        assert!(!Authorizer::check_all_permissions(
            &roles,
            &[Permission::ReadSBOM, Permission::ManageUsers]
        ));
    }

    #[test]
    fn test_authorizer_multiple_roles() {
        let roles = vec![Role::User, Role::CI];

        assert!(Authorizer::check_permission(&roles, &Permission::ReadSBOM));
        assert!(Authorizer::check_permission(&roles, &Permission::WriteSBOM)); // From CI role
    }

    #[test]
    fn test_is_admin() {
        assert!(Authorizer::is_admin(&[Role::Admin]));
        assert!(!Authorizer::is_admin(&[Role::User]));
        assert!(Authorizer::is_admin(&[Role::User, Role::Admin]));
    }
}
