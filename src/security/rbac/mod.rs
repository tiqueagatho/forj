use crate::error::{Error, Result};
use std::collections::HashSet;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Permission {
    Read(String),
    Write(String),
    Configure,
    Deploy,
    Audit,
}

pub struct RbacRole {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

pub struct RbacEngine {
    roles: Vec<RbacRole>,
}

impl RbacEngine {
    pub fn new() -> Self {
        Self { roles: vec![] }
    }

    pub fn add_role(&mut self, name: &str, perms: Vec<Permission>) {
        self.roles.push(RbacRole {
            name: name.into(),
            permissions: perms.into_iter().collect(),
        });
    }

    pub fn check(&self, user_roles: &[String], required: &Permission) -> Result<()> {
        for role_name in user_roles {
            if let Some(role) = self.roles.iter().find(|r| r.name == *role_name) {
                if role.permissions.contains(required) {
                    return Ok(());
                }
            }
        }
        Err(Error::Authorization {
            res: format!("{:?}", required),
            perm: format!("{:?}", required),
        })
    }
}

impl Default for RbacEngine {
    fn default() -> Self {
        Self::new()
    }
}
