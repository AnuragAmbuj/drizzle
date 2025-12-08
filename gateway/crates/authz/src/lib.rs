use anyhow::Result;
use async_trait::async_trait;
use authn::Identity;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request, Response,
};
use std::str::FromStr;
use std::sync::Arc;

#[async_trait]
pub trait PolicyEnforcer: Send + Sync {
    async fn authorize(
        &self,
        identity: Option<&Identity>,
        action: &str,
        resource: &str,
    ) -> Result<bool>;
}

use arc_swap::ArcSwap;
use domain::policy::Policy;

pub struct CedarPolicyEnforcer {
    authorizer: Authorizer,
    policies: ArcSwap<PolicySet>,
}

impl CedarPolicyEnforcer {
    pub fn new() -> Self {
        // Start with empty or static default
        let policies = PolicySet::default();
        Self {
            authorizer: Authorizer::new(),
            policies: ArcSwap::from_pointee(policies),
        }
    }

    pub fn with_policies(policies: PolicySet) -> Self {
        Self {
            authorizer: Authorizer::new(),
            policies: ArcSwap::from_pointee(policies),
        }
    }

    pub fn update_policies(&self, policies_domain: &[Policy]) {
        let mut new_set = PolicySet::new();

        for p in policies_domain {
            // Cedar policy ID must be unique
            match cedar_policy::Policy::parse(Some(p.id.to_string()), &p.content) {
                Ok(cedar_p) => {
                    if let Err(e) = new_set.add(cedar_p) {
                        eprintln!("Failed to add policy {}: {}", p.id, e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse policy {}: {}", p.id, e);
                }
            }
        }

        self.policies.store(Arc::new(new_set));
        eprintln!("Updated AuthZ policies. Count: {}", policies_domain.len());
    }
}

#[async_trait]
impl PolicyEnforcer for CedarPolicyEnforcer {
    async fn authorize(
        &self,
        identity: Option<&Identity>,
        action: &str,
        resource: &str,
    ) -> Result<bool> {
        let principal = if let Some(id) = identity {
            // Assuming subject is a valid EntityUid string or we construct one
            // E.g., User::"uuid"
            match EntityUid::from_str(&format!("User::\"{}\"", id.subject)) {
                Ok(uid) => Some(uid),
                Err(_) => {
                    // Fallback or explicit Anonymous
                    None
                }
            }
        } else {
            None
        };

        let action_uid = EntityUid::from_str(&format!("Action::\"{}\"", action))
            .map_err(|e| anyhow::anyhow!("Invalid action UID: {}", e))?;

        let resource_uid = EntityUid::from_str(&format!("Resource::\"{}\"", resource))
            .map_err(|e| anyhow::anyhow!("Invalid resource UID: {}", e))?;

        let context = Context::empty(); // Add context from request if needed later

        let request = Request::new(
            principal,
            Some(action_uid),
            Some(resource_uid),
            context,
            None, // Schema
        )
        .map_err(|e| anyhow::anyhow!("Validation error: {}", e))?;

        let entities = Entities::empty();
        let policies = self.policies.load();
        let response = self
            .authorizer
            .is_authorized(&request, &policies, &entities);

        match response.decision() {
            Decision::Allow => Ok(true),
            Decision::Deny => Ok(false),
        }
    }
}
