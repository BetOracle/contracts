# BetOracle Smart Contracts

Soroban-based smart contracts for the BetOracle prediction agent on Stellar.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    BetOracleFactory                        │
│         (Deploys and links all contracts)                │
└─────────────────────────────────────────────────────────┘
                           │
           ┌───────────────┴───────────────┐
           ▼                               ▼
┌─────────────────────┐      ┌──────────────────────────┐
│ BetOracleAgentWallet│◄────►│  BetOraclePrediction     │
│   (Agent Wallet)    │      │  (Prediction Registry)   │
└─────────────────────┘      └──────────────────────────┘
```

## Contracts

### BetOracleAgentWallet
Agent wallet contract that:
- Holds funds for agent operations
- Authorizes backends to submit predictions
- Tracks reputation on-chain
- Processes payments

### BetOraclePrediction
Prediction registry that:
- Stores all agent predictions immutably
- Tracks accuracy and reputation
- Resolves predictions after matches
- Provides query functions for frontend

### BetOracleFactory
Deployment factory that:
- Deploys agent wallet + prediction contract in one tx
- Tracks all deployed agents
- Simplifies deployment process

## Quick Start

### 1. Install dependencies
```bash
cd soroban-hello-world
cargo install soroban-cli
```

### 2. Build contracts
```bash
cd contracts/hello-world
cargo build --target wasm32-unknown-unknown --release
```

### 3. Run tests
```bash
cargo test
```

### 4. Deploy to Stellar Testnet
```bash
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/hello_world.wasm --source <SECRET_KEY> --network testnet
```

### 5. Deploy to Stellar Mainnet
```bash
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/hello_world.wasm --source <SECRET_KEY> --network pubnet
```

## Environment Variables

```bash
SECRET_KEY=S...                    # Your Stellar secret key
NETWORK=testnet                    # testnet or pubnet
BACKEND_ADDRESS=G...              # Backend wallet to authorize
```

## Contract Addresses (After Deployment)

After deployment, save these to `backend/.env`:

```bash
AGENT_ID=...              # From deployment output
AGENT_WALLET=G...        # Agent wallet address
PREDICTION_CONTRACT=G... # Prediction registry address
FACTORY=G...             # Factory address (optional)
```

## Testing

```bash
# Run all tests
cargo test

# Run with verbosity
cargo test -- --nocapture

# Run specific test
cargo test test_submit_prediction
```

## Stellar Resources

- Stellar Testnet Faucet: https://friendbot.stellar.org
- Stellar Expert: https://stellar.expert
- Stellar Docs: https://developers.stellar.org
- Soroban Docs: https://developers.stellar.org/docs/build/smart-contracts

## Integration with Backend

After deployment:

1. Copy contract addresses to `backend/.env`
2. Fund agent wallet with XLM for gas
3. Authorize backend address in agent wallet
4. Backend can now submit predictions via agent wallet

See `backend/README.md` for Python integration details.

## Deployment & Verification Checklist (for future redeployments)

### 1. Prerequisites
- Funded deployer wallet on Stellar mainnet (XLM for gas)
- `SECRET_KEY` set for deployment
- Soroban CLI installed

### 2. Environment setup
```bash
cd soroban-hello-world/contracts/hello-world
# Edit environment with:
# SECRET_KEY=<your Stellar secret key>
# BACKEND_ADDRESS=<backend address to authorize>
```

### 3. Build and test
```bash
cargo build --target wasm32-unknown-unknown --release
cargo test
```

### 4. Deploy to Stellar testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source $SECRET_KEY \
  --network testnet
```

### 5. Deploy to Stellar mainnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source $SECRET_KEY \
  --network pubnet
```

Save the output:
- Factory address
- Agent Wallet address
- Prediction Contract address
- Agent ID

### 6. Initialize contracts
After deployment, initialize each contract:

```bash
# Initialize Factory
soroban contract invoke \
  --id <FACTORY_ADDRESS> \
  --source $SECRET_KEY \
  --network pubnet \
  -- \
  initialize

# Initialize Agent Wallet
soroban contract invoke \
  --id <AGENT_WALLET_ADDRESS> \
  --source $SECRET_KEY \
  --network pubnet \
  -- \
  initialize

# Initialize Prediction Contract
soroban contract invoke \
  --id <PREDICTION_CONTRACT_ADDRESS> \
  --source $SECRET_KEY \
  --network pubnet \
  -- \
  initialize
```

### 7. Authorize backend address on Agent Wallet
```bash
soroban contract invoke \
  --id <AGENT_WALLET_ADDRESS> \
  --source $SECRET_KEY \
  --network pubnet \
  -- \
  authorize_backend \
  --backend $BACKEND_ADDRESS \
  --authorized true
```

### 8. Update Railway backend env vars
```bash
BLOCKCHAIN_ENABLED=true
AGENT_WALLET=<agent-wallet-address>
PREDICTION_CONTRACT=<prediction-contract-address>
STELLAR_RPC_URL=https://rpc.mainnet.stellar.org
AGENT_SECRET_KEY=<backend signer key>
```

### 9. Verify end-to-end
```bash
curl -X POST https://<railway-backend-url>/api/predict \
  -H "Content-Type: application/json" \
  -d '{"homeTeam":"Arsenal","awayTeam":"Chelsea","league":"EPL"}'
# Expect response with blockchain.submitted=true and txHash
```

### 10. Clean build artifacts
```bash
cargo clean
cargo build --target wasm32-unknown-unknown --release
```