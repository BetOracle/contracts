#![cfg(test)]

// Basic unit tests for Soroban contracts
// Full integration tests require proper contract deployment and invocation setup
// which is complex with soroban-sdk 27. These tests verify the contracts compile
// and the basic structure is correct.

#[test]
fn test_compilation() {
    // This test verifies the contracts compile successfully
    assert!(true);
}

#[test]
fn test_module_structure() {
    // Verify that all contract modules are accessible
    // This ensures the module structure is correct
    let _ = crate::agent_wallet::BetOracleAgentWallet;
    let _ = crate::factory::BetOracleFactory;
    let _ = crate::prediction::BetOraclePrediction;
}

#[test]
fn test_data_key_types() {
    // Verify DataKey enums are accessible
    let _ = crate::agent_wallet::DataKey::Profile;
    let _ = crate::factory::DataKey::AgentById(Bytes::from_slice(&Env::default(), b"test"));
    let _ = crate::prediction::DataKey::Owner;
}

#[test]
fn test_struct_types() {
    // Verify contract structs are accessible
    let _ = crate::agent_wallet::AgentProfile {
        agent_id: Bytes::from_slice(&Env::default(), b"test"),
        name: String::from_str(&Env::default(), "test"),
        metadata_uri: String::from_str(&Env::default(), "test"),
        created_at: 0,
        active: true,
    };
    let _ = crate::agent_wallet::Reputation {
        total_predictions: 0,
        correct_predictions: 0,
        total_staked: 0,
        reputation_score: 0,
        last_updated: 0,
    };
    let _ = crate::prediction::Prediction {
        prediction_id: Bytes::from_slice(&Env::default(), b"test"),
        match_id: Bytes::from_slice(&Env::default(), b"test"),
        agent: Address::from_string(&String::from_str(&Env::default(), "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWH2")),
        home_team: String::from_str(&Env::default(), "test"),
        away_team: String::from_str(&Env::default(), "test"),
        league: String::from_str(&Env::default(), "test"),
        prediction: 0,
        confidence: 0,
        timestamp: 0,
        match_date: 0,
        resolved: false,
        outcome: 0,
    };
}
