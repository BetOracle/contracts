use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, Map, String, Vec};
use soroban_sdk::contracterror;

#[derive(Clone)]
#[contracttype]
pub struct AgentProfile {
    pub agent_id: Bytes,
    pub name: String,
    pub metadata_uri: String,
    pub created_at: u64,
    pub active: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct Reputation {
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub total_staked: u64,
    pub reputation_score: u64, // 0-10000 (0-100% with 2 decimals)
    pub last_updated: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Profile,
    Reputation,
    Owner,
    PredictionContract,
    ProcessedPayment(Bytes),
    AuthorizedBackend(Address),
}

#[contract]
pub struct BetOracleAgentWallet;

#[contractimpl]
impl BetOracleAgentWallet {
    pub fn initialize(
        env: &Env,
        agent_id: Bytes,
        name: String,
        metadata_uri: String,
        owner: Address,
    ) {
        require!(!agent_id.is_empty(), "Invalid agent ID");
        require!(!name.is_empty(), "Name required");

        let profile = AgentProfile {
            agent_id: agent_id.clone(),
            name,
            metadata_uri,
            created_at: env.ledger().timestamp(),
            active: true,
        };

        env.storage().instance().set(&DataKey::Profile, &profile);
        env.storage().instance().set(&DataKey::Owner, &owner);

        env.events()
            .publish(("agent_registered", agent_id.clone()), (profile.name.clone(), owner, env.ledger().timestamp()));
    }

    pub fn authorize_backend(env: &Env, backend: Address, authorized: bool) {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Not authorized");

        if authorized {
            env.storage().instance().set(&DataKey::AuthorizedBackend(backend.clone()), &true);
        } else {
            env.storage().instance().remove(&DataKey::AuthorizedBackend(backend.clone()));
        }

        env.events()
            .publish(("backend_authorized", backend.clone()), authorized);
    }

    pub fn set_prediction_contract(env: &Env, contract: Address) {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Not authorized");
        require!(!contract.is_zero(), "Invalid contract");

        env.storage().instance().set(&DataKey::PredictionContract, &contract);
    }

    pub fn update_metadata(env: &Env, metadata_uri: String) {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Not authorized");

        let mut profile: AgentProfile = env.storage().instance().get::<DataKey, AgentProfile>(&DataKey::Profile).unwrap().unwrap();
        profile.metadata_uri = metadata_uri;
        env.storage().instance().set(&DataKey::Profile, &profile);
    }

    pub fn set_active(env: &Env, active: bool) {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Not authorized");

        let mut profile: AgentProfile = env.storage().instance().get::<DataKey, AgentProfile>(&DataKey::Profile).unwrap().unwrap();
        profile.active = active;
        env.storage().instance().set(&DataKey::Profile, &profile);
    }

    pub fn submit_prediction(
        env: &Env,
        prediction_id: Bytes,
        match_id: Bytes,
        home_team: String,
        away_team: String,
        league: String,
        prediction: u8,
        confidence: u64,
        match_date: u64,
    ) -> bool {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        let is_authorized = env.storage().instance().has(&DataKey::AuthorizedBackend(env.invoker()))
            || env.invoker() == owner;
        require!(is_authorized, "Not authorized");

        let prediction_contract: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::PredictionContract).unwrap().unwrap();
        require!(!prediction_contract.is_zero(), "Prediction contract not set");

        let profile: AgentProfile = env.storage().instance().get::<DataKey, AgentProfile>(&DataKey::Profile).unwrap().unwrap();
        require!(profile.active, "Agent not active");

        // Call prediction contract
        // Note: In Soroban, cross-contract calls are done via client.invoke_contract
        // This is a simplified version - actual implementation would use soroban-sdk's invoke_contract
        let mut reputation: Reputation = env.storage().instance().get(&DataKey::Reputation).unwrap().unwrap_or(Reputation {
            total_predictions: 0,
            correct_predictions: 0,
            total_staked: 0,
            reputation_score: 0,
            last_updated: 0,
        });

        reputation.total_predictions += 1;
        reputation.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Reputation, &reputation);

        env.events()
            .publish(("prediction_submitted", prediction_id.clone(), match_id.clone()), (confidence, env.ledger().timestamp()));

        true
    }

    pub fn update_reputation(env: &Env, total_predictions: u64, correct_predictions: u64) {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Not authorized");

        let mut reputation: Reputation = env.storage().instance().get(&DataKey::Reputation).unwrap().unwrap_or(Reputation {
            total_predictions: 0,
            correct_predictions: 0,
            total_staked: 0,
            reputation_score: 0,
            last_updated: 0,
        });

        reputation.total_predictions = total_predictions;
        reputation.correct_predictions = correct_predictions;

        if total_predictions > 0 {
            reputation.reputation_score = (correct_predictions * 10000) / total_predictions;
        }

        reputation.last_updated = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Reputation, &reputation);

        env.events()
            .publish(("reputation_updated", reputation.reputation_score), (total_predictions, correct_predictions, env.ledger().timestamp()));
    }

    pub fn process_payment(
        env: &Env,
        payment_id: Bytes,
        token: Address,
        amount: u64,
        recipient: Address,
    ) -> bool {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        let is_authorized = env.storage().instance().has(&DataKey::AuthorizedBackend(env.invoker()))
            || env.invoker() == owner;
        require!(is_authorized, "Not authorized");

        require!(!env.storage().instance().has(&DataKey::ProcessedPayment(payment_id.clone())), "Payment already processed");
        require!(!recipient.is_zero(), "Invalid recipient");
        require!(amount > 0, "Amount must be > 0");

        // Mark as processed first (re-entrancy guard)
        env.storage().instance().set(&DataKey::ProcessedPayment(payment_id.clone()), &true);

        // Transfer tokens
        // Note: In Soroban, token transfers are done via token contract
        // This is a simplified version - actual implementation would use soroban-token SDK
        if token.is_zero() {
            // Native token (XLM)
            // In Soroban, native token transfers are handled differently
            // This would require actual implementation with soroban-token
        } else {
            // Token transfer
            // This would require actual implementation with soroban-token
        }

        env.events()
            .publish(("payment_processed", payment_id.clone(), token.clone()), (amount, env.ledger().timestamp()));

        true
    }

    pub fn is_payment_processed(env: &Env, payment_id: Bytes) -> bool {
        env.storage().instance().has(&DataKey::ProcessedPayment(payment_id))
    }

    pub fn get_profile(env: &Env) -> AgentProfile {
        env.storage().instance().get(&DataKey::Profile).unwrap().unwrap()
    }

    pub fn get_reputation(env: &Env) -> Reputation {
        env.storage().instance().get(&DataKey::Reputation).unwrap().unwrap_or(Reputation {
            total_predictions: 0,
            correct_predictions: 0,
            total_staked: 0,
            reputation_score: 0,
            last_updated: 0,
        })
    }

    pub fn is_authorized(env: &Env, addr: Address) -> bool {
        let owner: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::Owner).unwrap().unwrap();
        env.storage().instance().has(&DataKey::AuthorizedBackend(addr.clone())) || addr == owner
    }

    pub fn get_balance(env: &Env, token: Address) -> u64 {
        // Note: In Soroban, balance checks are done via token contract
        // This is a placeholder - actual implementation would use soroban-token SDK
        0
    }
}
