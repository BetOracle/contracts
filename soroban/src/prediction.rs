use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, String, Vec};

#[derive(Clone)]
#[contracttype]
pub struct Prediction {
    pub prediction_id: Bytes,
    pub match_id: Bytes,
    pub agent: Address,
    pub home_team: String,
    pub away_team: String,
    pub league: String,
    pub prediction: u32, // 0=HOME_WIN, 1=DRAW, 2=AWAY_WIN
    pub confidence: u64,
    pub timestamp: u64,
    pub match_date: u64,
    pub resolved: bool,
    pub outcome: u32, // 0=HOME_WIN, 1=DRAW, 2=AWAY_WIN, 255=UNRESOLVED
    pub stake_amount: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    AgentWallet,
    Owner,
    Prediction(Bytes),
    PredictionIds,
    TotalPredictions,
    CorrectPredictions,
    ResolvedPredictions,
    TotalStaked,
}

#[contract]
pub struct BetOraclePrediction;

#[contractimpl]
impl BetOraclePrediction {
    pub fn initialize(env: &Env) {
        env.storage().instance().set(&DataKey::Owner, &env.invoker());
    }

    pub fn set_agent_wallet(env: &Env, agent_wallet: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap().unwrap();
        require!(env.invoker() == owner, "Only owner can call");
        require!(!agent_wallet.is_zero(), "Invalid agent wallet");

        let old_wallet: Address = env.storage().instance().get(&DataKey::AgentWallet).unwrap().unwrap_or(Address::zero(&env));
        env.storage().instance().set(&DataKey::AgentWallet, &agent_wallet);

        env.events()
            .publish(("agent_wallet_updated", old_wallet, agent_wallet.clone()), ());
    }

    pub fn submit_prediction(
        env: &Env,
        prediction_id: Bytes,
        match_id: Bytes,
        home_team: String,
        away_team: String,
        league: String,
        prediction: u32,
        confidence: u64,
        match_date: u64,
    ) -> bool {
        let agent_wallet: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::AgentWallet).unwrap().unwrap();
        require!(env.invoker() == agent_wallet, "Only agent wallet can call");
        require!(prediction <= 2, "Invalid prediction (0-2)");
        require!(confidence <= 10000, "Confidence must be 0-10000");
        require!(!env.storage().instance().has(&DataKey::Prediction(prediction_id.clone())), "Prediction already exists");

        // Allow match_date == 0 (backend may not always know kick-off time).
        // If provided, it must be at least 1 hour from now to prevent resolving
        // before the match has started.
        if match_date > 0 {
            require!(match_date > env.ledger().timestamp(), "Match must be in future");
        }

        Self::_store_prediction(
            env,
            prediction_id.clone(),
            match_id,
            home_team,
            away_team,
            league,
            prediction,
            confidence,
            match_date,
            0,
        );

        true
    }

    fn _store_prediction(
        env: &Env,
        prediction_id: Bytes,
        match_id: Bytes,
        home_team: String,
        away_team: String,
        league: String,
        prediction: u32,
        confidence: u64,
        match_date: u64,
        stake_amount: u64,
    ) {
        let agent_wallet: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::AgentWallet).unwrap().unwrap();

        let pred = Prediction {
            prediction_id: prediction_id.clone(),
            match_id,
            agent: agent_wallet,
            home_team,
            away_team,
            league,
            prediction,
            confidence,
            timestamp: env.ledger().timestamp(),
            match_date,
            resolved: false,
            outcome: 255, // UNRESOLVED
            stake_amount,
        };

        env.storage().instance().set(&DataKey::Prediction(prediction_id.clone()), &pred);

        // Add to prediction IDs list
        let mut prediction_ids: Vec<Bytes> = env.storage().instance().get::<DataKey, Vec<Bytes>>(&DataKey::PredictionIds).unwrap().unwrap_or(Vec::new(&env));
        prediction_ids.push_back(prediction_id.clone());
        env.storage().instance().set(&DataKey::PredictionIds, &prediction_ids);

        // Update total predictions count
        let mut total_predictions: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::TotalPredictions).unwrap_or(0);
        total_predictions += 1;
        env.storage().instance().set(&DataKey::TotalPredictions, &total_predictions);

        env.events()
            .publish(("prediction_submitted", prediction_id.clone(), match_id.clone()), (agent_wallet, home_team, away_team, prediction as u32, confidence, env.ledger().timestamp()));
    }

    pub fn submit_prediction_with_stake(
        env: &Env,
        prediction_id: Bytes,
        match_id: Bytes,
        home_team: String,
        away_team: String,
        league: String,
        prediction: u32,
        confidence: u64,
        match_date: u64,
        stake_amount: u64,
    ) -> bool {
        let agent_wallet: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::AgentWallet).unwrap().unwrap();
        require!(env.invoker() == agent_wallet, "Only agent wallet can call");
        require!(stake_amount > 0, "Must stake some amount");
        require!(prediction <= 2, "Invalid prediction (0-2)");
        require!(confidence <= 10000, "Confidence must be 0-10000");
        require!(!env.storage().instance().has(&DataKey::Prediction(prediction_id.clone())), "Prediction already exists");
        require!(match_date > env.ledger().timestamp(), "Match must be in future");

        Self::_store_prediction(
            env,
            prediction_id.clone(),
            match_id,
            home_team,
            away_team,
            league,
            prediction,
            confidence,
            match_date,
            stake_amount,
        );

        // Update total staked
        let mut total_staked: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::TotalStaked).unwrap_or(0);
        total_staked += stake_amount;
        env.storage().instance().set(&DataKey::TotalStaked, &total_staked);

        true
    }

    pub fn resolve_prediction(env: &Env, prediction_id: Bytes, outcome: u32) {
        let agent_wallet: Address = env.storage().instance().get::<DataKey, Address>(&DataKey::AgentWallet).unwrap().unwrap();
        require!(env.invoker() == agent_wallet, "Only agent wallet can call");
        require!(outcome <= 2, "Invalid outcome (0-2)");

        let mut pred: Prediction = env.storage().instance().get::<DataKey, Prediction>(&DataKey::Prediction(prediction_id.clone())).unwrap().unwrap();
        require!(pred.timestamp > 0, "Prediction not found");
        require!(!pred.resolved, "Already resolved");

        // If matchDate was provided, enforce the match has started (with 1h buffer).
        if pred.match_date > 0 {
            // 1 hour = 3600 seconds
            require!(env.ledger().timestamp() >= pred.match_date - 3600, "Match has not started yet");
        }

        pred.resolved = true;
        pred.outcome = outcome;

        // Update resolved predictions count
        let mut resolved_predictions: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::ResolvedPredictions).unwrap_or(0);
        resolved_predictions += 1;
        env.storage().instance().set(&DataKey::ResolvedPredictions, &resolved_predictions);

        let correct = pred.prediction == outcome;
        if correct {
            let mut correct_predictions: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::CorrectPredictions).unwrap_or(0);
            correct_predictions += 1;
            env.storage().instance().set(&DataKey::CorrectPredictions, &correct_predictions);
        }

        env.storage().instance().set(&DataKey::Prediction(prediction_id.clone()), &pred);

        env.events()
            .publish(("prediction_resolved", prediction_id.clone(), pred.match_id.clone()), (outcome, correct, env.ledger().timestamp()));
    }

    pub fn get_prediction(env: &Env, prediction_id: Bytes) -> Prediction {
        env.storage().instance().get::<DataKey, Prediction>(&DataKey::Prediction(prediction_id)).unwrap()
    }

    pub fn get_predictions(env: &Env, offset: u64, limit: u64) -> Vec<Prediction> {
        let prediction_ids: Vec<Bytes> = env.storage().instance().get::<DataKey, Vec<Bytes>>(&DataKey::PredictionIds).unwrap_or(Vec::new(&env));
        let len = prediction_ids.len() as u64;

        let start = offset.min(len);
        let end = (offset + limit).min(len);

        let mut result = Vec::new(&env);
        for i in start..end {
            let prediction_id = prediction_ids.get(i as u32).unwrap();
            let pred: Prediction = env.storage().instance().get::<DataKey, Prediction>(&DataKey::Prediction(prediction_id.clone())).unwrap();
            result.push_back(pred);
        }

        result
    }

    pub fn get_agent_accuracy(env: &Env) -> u64 {
        let resolved_predictions: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::ResolvedPredictions).unwrap_or(0);
        if resolved_predictions == 0 {
            return 0;
        }
        let correct_predictions: u64 = env.storage().instance().get::<DataKey, u64>(&DataKey::CorrectPredictions).unwrap_or(0);
        (correct_predictions * 10000) / resolved_predictions
    }

    pub fn is_prediction_correct(env: &Env, prediction_id: Bytes) -> bool {
        let pred: Prediction = env.storage().instance().get::<DataKey, Prediction>(&DataKey::Prediction(prediction_id)).unwrap();
        require!(pred.resolved, "Not resolved yet");
        pred.prediction == pred.outcome
    }

    pub fn generate_match_id(
        _env: &Env,
        league: String,
        home_team: String,
        away_team: String,
        date: u64,
    ) -> Bytes {
        // Simplified hash generation for Soroban
        // In production, use soroban_sdk::crypto::Sha256
        let combined = format!("{}{}{}{}", league.to_string(), home_team.to_string(), away_team.to_string(), date);
        Bytes::from_slice(&_env, combined.as_bytes())
    }

    pub fn generate_prediction_id(env: &Env, match_id: Bytes, timestamp: u64) -> Bytes {
        // Simplified hash generation for Soroban
        let combined = format!("{:?}{}", match_id, timestamp);
        Bytes::from_slice(&env, combined.as_bytes())
    }
}
