//! User entity for authentication and authorization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::EntityId;

/// User roles for role-based access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Read-only access to all data
    Viewer,
    /// Standard lab technician - can create/edit samples, libraries, pools
    Technician,
    /// Lab manager - full access to lab operations, can delete
    LabManager,
    /// IT administrator - system configuration access
    Admin,
    /// Super user - full access to everything
    SuperAdmin,
}

impl Role {
    /// Returns true if this role has administrative privileges.
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin | Self::SuperAdmin)
    }

    /// Returns true if this role can modify lab data.
    pub fn can_edit(&self) -> bool {
        matches!(self, Self::Technician | Self::LabManager | Self::Admin | Self::SuperAdmin)
    }

    /// Returns true if this role can delete entities.
    pub fn can_delete(&self) -> bool {
        matches!(self, Self::LabManager | Self::Admin | Self::SuperAdmin)
    }

    /// Returns true if this role can manage users.
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin | Self::SuperAdmin)
    }

    /// Returns the permission level (higher = more permissions).
    pub fn level(&self) -> u8 {
        match self {
            Self::Viewer => 1,
            Self::Technician => 2,
            Self::LabManager => 3,
            Self::Admin => 4,
            Self::SuperAdmin => 5,
        }
    }

    /// Returns true if this role has at least the permissions of `other`.
    pub fn has_at_least(&self, other: &Role) -> bool {
        self.level() >= other.level()
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Viewer => write!(f, "Viewer"),
            Self::Technician => write!(f, "Technician"),
            Self::LabManager => write!(f, "Lab Manager"),
            Self::Admin => write!(f, "Administrator"),
            Self::SuperAdmin => write!(f, "Super Administrator"),
        }
    }
}

/// A user in the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    pub id: EntityId,
    /// Username (for login)
    pub username: String,
    /// Display name
    pub display_name: String,
    /// Email address
    pub email: String,
    /// User's role
    pub role: Role,
    /// Is the user account active?
    pub active: bool,
    /// Is this an internal (local) or external (LDAP) user?
    pub internal: bool,
    /// When the user was created
    pub created_at: DateTime<Utc>,
    /// When the user last logged in
    pub last_login_at: Option<DateTime<Utc>>,
    /// When the user was last modified
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new internal user.
    pub fn new_internal(
        id: EntityId,
        username: String,
        display_name: String,
        email: String,
        role: Role,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            username,
            display_name,
            email,
            role,
            active: true,
            internal: true,
            created_at: now,
            last_login_at: None,
            updated_at: now,
        }
    }

    /// Creates a new LDAP user.
    pub fn new_ldap(
        id: EntityId,
        username: String,
        display_name: String,
        email: String,
        role: Role,
    ) -> Self {
        let mut user = Self::new_internal(id, username, display_name, email, role);
        user.internal = false;
        user
    }

    /// Records a login.
    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Deactivates the user.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Activates the user.
    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = Utc::now();
    }

    /// Changes the user's role.
    pub fn set_role(&mut self, role: Role) {
        self.role = role;
        self.updated_at = Utc::now();
    }

    /// Returns true if this user can perform the action.
    pub fn can_edit(&self) -> bool {
        self.active && self.role.can_edit()
    }

    /// Returns true if this user can delete entities.
    pub fn can_delete(&self) -> bool {
        self.active && self.role.can_delete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        assert!(Role::SuperAdmin.has_at_least(&Role::Admin));
        assert!(Role::Admin.has_at_least(&Role::LabManager));
        assert!(Role::LabManager.has_at_least(&Role::Technician));
        assert!(Role::Technician.has_at_least(&Role::Viewer));
        assert!(!Role::Viewer.has_at_least(&Role::Technician));
    }

    #[test]
    fn test_role_permissions() {
        assert!(Role::SuperAdmin.can_delete());
        assert!(Role::LabManager.can_delete());
        assert!(!Role::Technician.can_delete());
        assert!(Role::Technician.can_edit());
        assert!(!Role::Viewer.can_edit());
    }

    #[test]
    fn test_user_creation() {
        let user = User::new_internal(
            1,
            "jdoe".to_string(),
            "John Doe".to_string(),
            "jdoe@example.com".to_string(),
            Role::Technician,
        );
        assert!(user.active);
        assert!(user.internal);
        assert!(user.can_edit());
        assert!(!user.can_delete());
    }

    #[test]
    fn test_user_deactivation() {
        let mut user = User::new_internal(
            1,
            "jdoe".to_string(),
            "John Doe".to_string(),
            "jdoe@example.com".to_string(),
            Role::LabManager,
        );

        assert!(user.can_delete());

        user.deactivate();
        assert!(!user.can_delete());
        assert!(!user.can_edit());
    }
}

