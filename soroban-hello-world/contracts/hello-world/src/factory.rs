use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, String, Vec};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    AgentWallets,
    AgentById(Bytes),
    PredictionContracts,
    AgentWalletCount,
    PredictionContractCount,
}

#[contract]
pub struct BetOracleFactory;

#[contractimpl]
impl BetOracleFactory {
    pub fn deploy_agent_wallet(
        env: &Env,
        agent_id: Bytes,
        name: String,
        metadata_uri: String,
    ) -> Address {
        let agent_wallet = Self::_deploy_agent_wallet_internal(env, agent_id, name, metadata_uri);
        agent_wallet
    }

    fn _deploy_agent_wallet_internal(
        env: &Env,
        agent_id: Bytes,
        name: String,
        metadata_uri: String,
    ) -> Address {
        if env.storage().instance().has(&DataKey::AgentById(agent_id.clone())) {
            panic!("Agent ID already exists");
        }

        // In Soroban, contract deployment is done via deployer
        // For this conversion, we use the invoker's address as a placeholder
        // In production, this would deploy a new BetOracleAgentWallet contract
        let agent_wallet = env.invoker();

        // Add to agent wallets list
        let mut agent_wallets: Vec<Address> = if env.storage().instance().has(&DataKey::AgentWallets) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::AgentWallets).unwrap()
        } else {
            Vec::new(&env)
        };
        agent_wallets.push_back(agent_wallet.clone());
        env.storage().instance().set(&DataKey::AgentWallets, &agent_wallets);

        // Map agent ID to wallet
        env.storage().instance().set(&DataKey::AgentById(agent_id.clone()), &agent_wallet);

        env.events()
            .publish(("agent_deployed", agent_id.clone(), agent_wallet.clone()), (name.clone(), env.invoker(), env.ledger().timestamp()));

        agent_wallet
    }

    pub fn deploy_prediction_contract(env: &Env, agent_wallet: Address) -> Address {
        let prediction_contract = Self::_deploy_prediction_contract_internal(env, agent_wallet);
        prediction_contract
    }

    fn _deploy_prediction_contract_internal(env: &Env, agent_wallet: Address) -> Address {
        // In Soroban, contract deployment is done via deployer
        // For this conversion, we use the invoker's address as a placeholder
        // In production, this would deploy a new BetOraclePrediction contract
        let prediction_contract = env.invoker();

        // Add to prediction contracts list
        let mut prediction_contracts: Vec<Address> = if env.storage().instance().has(&DataKey::PredictionContracts) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::PredictionContracts).unwrap()
        } else {
            Vec::new(&env)
        };
        prediction_contracts.push_back(prediction_contract.clone());
        env.storage().instance().set(&DataKey::PredictionContracts, &prediction_contracts);

        // Set agent wallet on prediction contract
        // Note: This would require cross-contract call to prediction contract
        // This is a placeholder - actual implementation would use soroban-sdk's invoke_contract

        env.events()
            .publish(("prediction_contract_deployed", prediction_contract.clone(), agent_wallet.clone()), env.ledger().timestamp());

        prediction_contract
    }

    pub fn deploy_full(
        env: &Env,
        agent_id: Bytes,
        name: String,
        metadata_uri: String,
    ) -> (Address, Address) {
        // Deploy agent
        let agent_wallet = Self::_deploy_agent_wallet_internal(env, agent_id.clone(), name.clone(), metadata_uri);

        // Deploy prediction contract
        let prediction_contract = Self::_deploy_prediction_contract_internal(env, agent_wallet.clone());

        // Note: Caller (owner of agent wallet) must call set_prediction_contract()
        // on agent wallet to complete the linkage

        env.events()
            .publish(("full_deployment", agent_id.clone(), agent_wallet.clone(), prediction_contract.clone()), (env.invoker(), env.ledger().timestamp()));

        (agent_wallet, prediction_contract)
    }

    pub fn get_agent_wallet(env: &Env, agent_id: Bytes) -> Address {
        env.storage().instance().get::<DataKey, Address>(&DataKey::AgentById(agent_id)).unwrap()
    }

    pub fn get_all_agent_wallets(env: &Env) -> Vec<Address> {
        if env.storage().instance().has(&DataKey::AgentWallets) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::AgentWallets).unwrap()
        } else {
            Vec::new(&env)
        }
    }

    pub fn get_all_prediction_contracts(env: &Env) -> Vec<Address> {
        if env.storage().instance().has(&DataKey::PredictionContracts) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::PredictionContracts).unwrap()
        } else {
            Vec::new(&env)
        }
    }

    pub fn get_deployment_count(env: &Env) -> (u64, u64) {
        let agent_wallets: Vec<Address> = if env.storage().instance().has(&DataKey::AgentWallets) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::AgentWallets).unwrap()
        } else {
            Vec::new(&env)
        };
        let prediction_contracts: Vec<Address> = if env.storage().instance().has(&DataKey::PredictionContracts) {
            env.storage().instance().get::<DataKey, Vec<Address>>(&DataKey::PredictionContracts).unwrap()
        } else {
            Vec::new(&env)
        };
        (agent_wallets.len() as u64, prediction_contracts.len() as u64)
    }
}
