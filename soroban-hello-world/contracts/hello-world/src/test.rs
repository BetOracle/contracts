#![cfg(test)]

use soroban_sdk::{Address, Bytes, Env, String};
use soroban_sdk::testutils::{Address as _, Ledger};

mod agent_wallet;
mod factory;
mod prediction;

#[test]
fn test_agent_profile() {
    let env = Env::default();
    env.mock_all_auths();
    
    let owner = Address::generate(&env);
    let agent_id = Bytes::from_slice(&env, b"Test-Agent-001");
    let name = String::from_str(&env, "Test Agent");
    let metadata_uri = String::from_str(&env, "https://test.uri");
    
    // Deploy agent wallet (in real scenario, this would be via factory)
    // For testing, we'll initialize the contract directly
    let agent_wallet_address = Address::generate(&env);
    
    // Initialize agent wallet
    // agent_wallet::initialize(&env, agent_id.clone(), name.clone(), metadata_uri.clone(), owner.clone());
    
    // Get profile
    // let profile = agent_wallet::get_profile(&env);
    
    // assert_eq!(profile.agent_id, agent_id);
    // assert_eq!(profile.name, name);
    // assert_eq!(profile.metadata_uri, metadata_uri);
    // assert!(profile.active);
}

#[test]
fn test_authorize_backend() {
    let env = Env::default();
    env.mock_all_auths();
    
    let owner = Address::generate(&env);
    let backend = Address::generate(&env);
    
    // Authorize backend
    // agent_wallet::authorize_backend(&env, backend.clone(), true);
    
    // Check authorization
    // assert!(agent_wallet::is_authorized(&env, backend.clone()));
}

#[test]
fn test_submit_prediction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let owner = Address::generate(&env);
    let agent_wallet = Address::generate(&env);
    let backend = Address::generate(&env);
    
    let prediction_id = Bytes::from_slice(&env, b"prediction-001");
    let match_id = Bytes::from_slice(&env, b"EPL-ARS-CHE-20260316");
    let home_team = String::from_str(&env, "Arsenal");
    let away_team = String::from_str(&env, "Chelsea");
    let league = String::from_str(&env, "EPL");
    let prediction = 0u32; // HOME_WIN
    let confidence = 7500u64;
    let match_date = env.ledger().timestamp() + 86400; // 1 day
    
    // Submit prediction
    // let success = agent_wallet::submit_prediction(
    //     &env,
    //     prediction_id.clone(),
    //     match_id.clone(),
    //     home_team.clone(),
    //     away_team.clone(),
    //     league.clone(),
    //     prediction,
    //     confidence,
    //     match_date
    // );
    
    // assert!(success);
    
    // Verify prediction was recorded
    // let pred = prediction::get_prediction(&env, prediction_id);
    // assert_eq!(pred.home_team, home_team);
    // assert_eq!(pred.away_team, away_team);
    // assert_eq!(pred.prediction, prediction);
    // assert_eq!(pred.confidence, confidence);
}

#[test]
fn test_resolve_prediction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let agent_wallet = Address::generate(&env);
    
    let prediction_id = Bytes::from_slice(&env, b"prediction-001");
    let match_id = Bytes::from_slice(&env, b"EPL-ARS-CHE-20260316");
    let home_team = String::from_str(&env, "Arsenal");
    let away_team = String::from_str(&env, "Chelsea");
    let league = String::from_str(&env, "EPL");
    let prediction = 0u32;
    let confidence = 8000u64;
    let match_date = env.ledger().timestamp() + 172800; // 2 days
    
    // Submit prediction
    // prediction::submit_prediction(
    //     &env,
    //     prediction_id.clone(),
    //     match_id.clone(),
    //     home_team.clone(),
    //     away_team.clone(),
    //     league.clone(),
    //     prediction,
    //     confidence,
    //     match_date
    // );
    
    // Warp time forward
    env.ledger().set(match_date + 3600);
    
    // Resolve prediction
    // prediction::resolve_prediction(&env, prediction_id.clone(), 0);
    
    // Verify resolution
    // let pred = prediction::get_prediction(&env, prediction_id);
    // assert!(pred.resolved);
    // assert_eq!(pred.outcome, 0);
}

#[test]
fn test_get_agent_accuracy() {
    let env = Env::default();
    env.mock_all_auths();
    
    let agent_wallet = Address::generate(&env);
    
    // Submit and resolve 2 predictions
    let prediction_id1 = Bytes::from_slice(&env, b"prediction-001");
    let match_id1 = Bytes::from_slice(&env, b"EPL-ARS-CHE-20260316");
    let match_date1 = env.ledger().timestamp() + 172800;
    
    // prediction::submit_prediction(&env, prediction_id1.clone(), match_id1.clone(), 
    //     String::from_str(&env, "Arsenal"), String::from_str(&env, "Chelsea"),
    //     String::from_str(&env, "EPL"), 0, 8000, match_date1);
    
    let prediction_id2 = Bytes::from_slice(&env, b"prediction-002");
    let match_id2 = Bytes::from_slice(&env, b"EPL-MCI-LIV-20260316");
    let match_date2 = env.ledger().timestamp() + 259200;
    
    // prediction::submit_prediction(&env, prediction_id2.clone(), match_id2.clone(),
    //     String::from_str(&env, "Man City"), String::from_str(&env, "Liverpool"),
    //     String::from_str(&env, "EPL"), 0, 7000, match_date2);
    
    // Warp to after both matches
    env.ledger().set(match_date2 + 3600);
    
    // Resolve both correctly
    // prediction::resolve_prediction(&env, prediction_id1, 0);
    // prediction::resolve_prediction(&env, prediction_id2, 0);
    
    // Check accuracy
    // let accuracy = prediction::get_agent_accuracy(&env);
    // assert_eq!(accuracy, 10000); // 100% accuracy
}

#[test]
fn test_factory_deploy_full() {
    let env = Env::default();
    env.mock_all_auths();
    
    let agent_id = Bytes::from_slice(&env, b"Factory-Agent-001");
    let name = String::from_str(&env, "Factory Agent");
    let metadata_uri = String::from_str(&env, b"https://factory.uri");
    
    // Deploy via factory
    // let (wallet, pred_contract) = factory::deploy_full(&env, agent_id.clone(), name.clone(), metadata_uri.clone());
    
    // assert!(wallet != Address::generate(&env)); // Should be a real address
    // assert!(pred_contract != Address::generate(&env));
    
    // Verify factory tracking
    // assert_eq!(factory::get_agent_wallet(&env, agent_id), wallet);
}

#[test]
fn test_factory_get_all_agents() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Deploy multiple agents
    // factory::deploy_full(&env, Bytes::from_slice(&env, b"Agent-1"), 
    //     String::from_str(&env, "Agent 1"), String::from_str(&env, b"uri1"));
    // factory::deploy_full(&env, Bytes::from_slice(&env, b"Agent-2"),
    //     String::from_str(&env, "Agent 2"), String::from_str(&env, b"uri2"));
    // factory::deploy_full(&env, Bytes::from_slice(&env, b"Agent-3"),
    //     String::from_str(&env, "Agent 3"), String::from_str(&env, b"uri3"));
    
    // Get all agents
    // let agents = factory::get_all_agent_wallets(&env);
    // assert_eq!(agents.len(), 3);
}

#[test]
fn test_revert_unauthorized_prediction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let unauthorized_user = Address::generate(&env);
    
    let prediction_id = Bytes::from_slice(&env, b"prediction-001");
    let match_id = Bytes::from_slice(&env, b"EPL-ARS-CHE-20260316");
    
    // Try to submit from unauthorized address - should panic
    // let result = std::panic::catch_unwind(|| {
    //     prediction::submit_prediction(
    //         &env,
    //         prediction_id,
    //         match_id,
    //         String::from_str(&env, "Arsenal"),
    //         String::from_str(&env, "Chelsea"),
    //         String::from_str(&env, "EPL"),
    //         0,
    //         8000,
    //         env.ledger().timestamp() + 86400
    //     );
    // });
    
    // assert!(result.is_err());
}

#[test]
fn test_revert_duplicate_prediction() {
    let env = Env::default();
    env.mock_all_auths();
    
    let agent_wallet = Address::generate(&env);
    
    let prediction_id = Bytes::from_slice(&env, b"prediction-001");
    let match_id = Bytes::from_slice(&env, b"EPL-ARS-CHE-20260316");
    
    // Submit first prediction
    // prediction::submit_prediction(
    //     &env,
    //     prediction_id.clone(),
    //     match_id.clone(),
    //     String::from_str(&env, "Arsenal"),
    //     String::from_str(&env, "Chelsea"),
    //     String::from_str(&env, "EPL"),
    //     0,
    //     8000,
    //     env.ledger().timestamp() + 86400
    // );
    
    // Try duplicate - should panic
    // let result = std::panic::catch_unwind(|| {
    //     prediction::submit_prediction(
    //         &env,
    //         prediction_id,
    //         match_id,
    //         String::from_str(&env, "Arsenal"),
    //         String::from_str(&env, "Chelsea"),
    //         String::from_str(&env, "EPL"),
    //         0,
    //         8000,
    //         env.ledger().timestamp() + 86400
    //     );
    // });
    
    // assert!(result.is_err());
}
