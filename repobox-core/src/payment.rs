use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::{ConfigError, VirtualsConfig, VirtualsPaymentConfig};

/// Payment status for a bounty claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

/// A bounty payment claim record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BountyClaim {
    pub id: String,
    pub agent_address: String,
    pub issue_id: String,
    pub severity: String,
    pub amount_usdc: String,
    pub commit_hash: String,
    pub branch_name: String,
    pub pr_number: Option<u32>,
    pub status: PaymentStatus,
    pub transaction_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Payment processor for handling bounty claims.
#[derive(Debug, Clone)]
pub struct PaymentProcessor {
    pub config: VirtualsPaymentConfig,
}

impl PaymentProcessor {
    pub fn new(config: VirtualsPaymentConfig) -> Self {
        Self { config }
    }

    /// Create a bounty claim for a successfully merged agent PR.
    pub fn create_bounty_claim(
        &self,
        virtuals_config: &VirtualsConfig,
        agent_address: &str,
        issue_id: &str,
        severity: &str,
        commit_hash: &str,
        branch_name: &str,
        pr_number: Option<u32>,
    ) -> Result<BountyClaim, ConfigError> {
        // Determine bounty amount based on severity
        let amount_usdc = match severity {
            "critical" => &virtuals_config.bug_bounties.critical,
            "high" => &virtuals_config.bug_bounties.high,
            "medium" => &virtuals_config.bug_bounties.medium,
            "low" => &virtuals_config.bug_bounties.low,
            _ => return Err(ConfigError::InvalidRule(format!("unknown severity: {}", severity))),
        };

        let claim_id = generate_claim_id(agent_address, issue_id, commit_hash);
        let now = chrono::Utc::now();

        Ok(BountyClaim {
            id: claim_id,
            agent_address: agent_address.to_string(),
            issue_id: issue_id.to_string(),
            severity: severity.to_string(),
            amount_usdc: amount_usdc.clone(),
            commit_hash: commit_hash.to_string(),
            branch_name: branch_name.to_string(),
            pr_number,
            status: PaymentStatus::Pending,
            transaction_hash: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Process a payment for a bounty claim.
    /// In a real implementation, this would interact with the blockchain.
    pub fn process_payment(&self, claim: &mut BountyClaim) -> Result<(), ConfigError> {
        // Validate payment configuration
        self.validate_payment_config()?;

        // Set status to processing
        claim.status = PaymentStatus::Processing;
        claim.updated_at = chrono::Utc::now();

        // In a real implementation, this would:
        // 1. Connect to Base network RPC
        // 2. Create and sign USDC transfer transaction
        // 3. Submit to network and wait for confirmation
        // 4. Update claim with transaction hash

        // For now, simulate successful payment
        let mock_tx_hash = format!("0x{:064x}", rand::random::<u64>());
        claim.transaction_hash = Some(mock_tx_hash);
        claim.status = PaymentStatus::Completed;
        claim.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Validate that the payment configuration is correct.
    fn validate_payment_config(&self) -> Result<(), ConfigError> {
        // Validate network
        if !["base", "ethereum", "polygon", "arbitrum", "optimism"].contains(&self.config.network.as_str()) {
            return Err(ConfigError::InvalidRule(format!(
                "unsupported payment network: {}",
                self.config.network
            )));
        }

        // Validate treasury address format
        if !self.config.treasury.starts_with("0x") || self.config.treasury.len() != 42 {
            return Err(ConfigError::InvalidRule(format!(
                "invalid treasury address: {}",
                self.config.treasury
            )));
        }

        Ok(())
    }

    /// Get network RPC URL for the configured network.
    pub fn get_rpc_url(&self) -> &'static str {
        match self.config.network.as_str() {
            "base" => "https://mainnet.base.org",
            "ethereum" => "https://eth-mainnet.g.alchemy.com/v2",
            "polygon" => "https://polygon-mainnet.g.alchemy.com/v2",
            "arbitrum" => "https://arb-mainnet.g.alchemy.com/v2",
            "optimism" => "https://opt-mainnet.g.alchemy.com/v2",
            _ => "https://mainnet.base.org", // Default to Base
        }
    }

    /// Get USDC contract address for the configured network.
    pub fn get_usdc_contract_address(&self) -> &'static str {
        match self.config.network.as_str() {
            "base" => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // USDC on Base
            "ethereum" => "0xA0b86a33E6441b419C3fe7E54bDE74CA5C1b36a", // USDC on Ethereum
            "polygon" => "0x2791bca1f2de4661ed88a30c99a7a9449aa84174", // USDC on Polygon
            "arbitrum" => "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC on Arbitrum
            "optimism" => "0x0b2c639c533813f4aa9d7837caf62653d097ff85", // USDC on Optimism
            _ => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", // Default to Base
        }
    }
}

/// Generate a unique claim ID based on agent, issue, and commit.
fn generate_claim_id(agent_address: &str, issue_id: &str, commit_hash: &str) -> String {
    use sha3::{Digest, Keccak256};

    let input = format!("{}:{}:{}", agent_address, issue_id, commit_hash);
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("claim_{:016x}", u64::from_be_bytes(result[0..8].try_into().unwrap()))
}

/// Storage interface for bounty claims.
pub trait ClaimsStorage {
    fn save_claim(&mut self, claim: &BountyClaim) -> Result<(), ConfigError>;
    fn get_claim(&self, id: &str) -> Result<Option<BountyClaim>, ConfigError>;
    fn get_claims_by_agent(&self, agent_address: &str) -> Result<Vec<BountyClaim>, ConfigError>;
    fn get_pending_claims(&self) -> Result<Vec<BountyClaim>, ConfigError>;
    fn update_claim_status(&mut self, id: &str, status: PaymentStatus) -> Result<(), ConfigError>;
}

/// In-memory implementation of ClaimsStorage for testing.
#[derive(Debug, Default)]
pub struct MemoryClaimsStorage {
    claims: HashMap<String, BountyClaim>,
}

impl ClaimsStorage for MemoryClaimsStorage {
    fn save_claim(&mut self, claim: &BountyClaim) -> Result<(), ConfigError> {
        self.claims.insert(claim.id.clone(), claim.clone());
        Ok(())
    }

    fn get_claim(&self, id: &str) -> Result<Option<BountyClaim>, ConfigError> {
        Ok(self.claims.get(id).cloned())
    }

    fn get_claims_by_agent(&self, agent_address: &str) -> Result<Vec<BountyClaim>, ConfigError> {
        let claims: Vec<BountyClaim> = self.claims
            .values()
            .filter(|claim| claim.agent_address == agent_address)
            .cloned()
            .collect();
        Ok(claims)
    }

    fn get_pending_claims(&self) -> Result<Vec<BountyClaim>, ConfigError> {
        let claims: Vec<BountyClaim> = self.claims
            .values()
            .filter(|claim| matches!(claim.status, PaymentStatus::Pending))
            .cloned()
            .collect();
        Ok(claims)
    }

    fn update_claim_status(&mut self, id: &str, status: PaymentStatus) -> Result<(), ConfigError> {
        if let Some(claim) = self.claims.get_mut(id) {
            claim.status = status;
            claim.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(ConfigError::InvalidRule(format!("claim not found: {}", id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_virtuals_config() -> VirtualsConfig {
        use crate::config::{VirtualsConfig, BugBountyConfig, AgentRequirements, VirtualsPaymentConfig};

        VirtualsConfig {
            enabled: true,
            bug_bounties: BugBountyConfig {
                critical: "100.00".to_string(),
                high: "50.00".to_string(),
                medium: "25.00".to_string(),
                low: "10.00".to_string(),
            },
            agent_requirements: AgentRequirements {
                min_reputation: 0.8,
                required_tests: true,
                human_review_required: true,
            },
            payments: Some(VirtualsPaymentConfig {
                network: "base".to_string(),
                token: "USDC".to_string(),
                treasury: "0x1234567890123456789012345678901234567890".to_string(),
                gas_sponsor: None,
            }),
        }
    }

    #[test]
    fn test_create_bounty_claim() {
        let virtuals_config = create_test_virtuals_config();
        let payment_config = virtuals_config.payments.clone().unwrap();
        let processor = PaymentProcessor::new(payment_config);

        let claim = processor.create_bounty_claim(
            &virtuals_config,
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "42",
            "high",
            "abcd1234",
            "agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42",
            Some(123),
        ).unwrap();

        assert_eq!(claim.agent_address, "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00");
        assert_eq!(claim.issue_id, "42");
        assert_eq!(claim.severity, "high");
        assert_eq!(claim.amount_usdc, "50.00");
        assert_eq!(claim.commit_hash, "abcd1234");
        assert_eq!(claim.pr_number, Some(123));
        assert!(matches!(claim.status, PaymentStatus::Pending));
    }

    #[test]
    fn test_payment_processor_network_configs() {
        let payment_config = VirtualsPaymentConfig {
            network: "base".to_string(),
            token: "USDC".to_string(),
            treasury: "0x1234567890123456789012345678901234567890".to_string(),
            gas_sponsor: None,
        };
        let processor = PaymentProcessor::new(payment_config);

        assert_eq!(processor.get_rpc_url(), "https://mainnet.base.org");
        assert_eq!(processor.get_usdc_contract_address(), "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
    }

    #[test]
    fn test_memory_claims_storage() {
        let mut storage = MemoryClaimsStorage::default();
        let virtuals_config = create_test_virtuals_config();
        let payment_config = virtuals_config.payments.clone().unwrap();
        let processor = PaymentProcessor::new(payment_config);

        let claim = processor.create_bounty_claim(
            &virtuals_config,
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "42",
            "medium",
            "abcd1234",
            "agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42",
            None,
        ).unwrap();

        // Save claim
        storage.save_claim(&claim).unwrap();

        // Retrieve claim
        let retrieved = storage.get_claim(&claim.id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().amount_usdc, "25.00");

        // Get claims by agent
        let agent_claims = storage.get_claims_by_agent("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00").unwrap();
        assert_eq!(agent_claims.len(), 1);

        // Get pending claims
        let pending_claims = storage.get_pending_claims().unwrap();
        assert_eq!(pending_claims.len(), 1);

        // Update status
        storage.update_claim_status(&claim.id, PaymentStatus::Completed).unwrap();
        let pending_after_update = storage.get_pending_claims().unwrap();
        assert_eq!(pending_after_update.len(), 0);
    }

    #[test]
    fn test_generate_claim_id_deterministic() {
        let id1 = generate_claim_id("0xabc", "123", "hash1");
        let id2 = generate_claim_id("0xabc", "123", "hash1");
        let id3 = generate_claim_id("0xabc", "123", "hash2");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1.starts_with("claim_"));
    }
}