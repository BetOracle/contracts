# BetOracle Smart Contracts

Foundry-based smart contracts for the BetOracle prediction agent on Celo.

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
│   (ERC-8004)        │      │  (Prediction Registry)   │
└─────────────────────┘      └──────────────────────────┘
```

## Contracts

### BetOracleAgentWallet
ERC-8004 compliant agent wallet that:
- Holds funds for agent operations
- Authorizes backends to submit predictions
- Tracks reputation on-chain
- Processes x402 payments

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
cd contracts
forge install
```

### 2. Set up environment
```bash
cp .env.example .env
# Edit .env with your PRIVATE_KEY and RPC_URL
```

### 3. Run tests
```bash
forge test
```

### 4. Deploy to Celo Alfajores (testnet)
```bash
source .env
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
```

### 5. Deploy to Celo Mainnet
```bash
source .env
forge script script/Deploy.s.sol --rpc-url https://forno.celo.org --broadcast
```

## Environment Variables

```bash
PRIVATE_KEY=0x...                    # Your wallet private key
RPC_URL=https://alfajores-forno...   # Celo Alfajores or Mainnet
CELOSCAN_API_KEY=...                 # For contract verification
BACKEND_ADDRESS=0x...                # Backend wallet to authorize
```

## Contract Addresses (After Deployment)

After deployment, save these to `backend/.env`:

```bash
AGENT_ID=0x...           # From deployment output
AGENT_WALLET=0x...       # Agent wallet address
PREDICTION_CONTRACT=0x... # Prediction registry address
FACTORY=0x...            # Factory address (optional)
```

## Testing

```bash
# Run all tests
forge test

# Run with verbosity
forge test -vv

# Run specific test
forge test --match-test test_SubmitPrediction

# Gas report
forge test --gas-report
```

## Celo Resources

- Celo Alfajores Faucet: https://faucet.celo.org
- CeloScan: https://celoscan.io
- Celo Docs: https://docs.celo.org

## Integration with Backend

After deployment:

1. Copy contract addresses to `backend/.env`
2. Fund agent wallet with CELO for gas
3. Authorize backend address in agent wallet
4. Backend can now submit predictions via agent wallet

See `backend/README.md` for Python integration details.

## Deployment & Verification Checklist (for future redeployments)

### 1. Prerequisites
- Funded deployer wallet on Celo mainnet (CELO for gas)
- `PRIVATE_KEY` set in `contracts/.env` (without `0x` prefix)
- Optional: `ETHERSCAN_API_KEY` for contract verification (Etherscan API v2, chainid=42220)

### 2. Environment setup
```bash
cd contracts
cp .env.example .env
# Edit .env with:
# PRIVATE_KEY=<64-hex-without-0x>
# BACKEND_ADDRESS=<backend EOA to authorize>
# ETHERSCAN_API_KEY=<your key>
```

### 3. Build and test (optional)
```bash
forge build
forge test
```

### 4. Simulation (dry-run, no gas)
```bash
set -a; source .env; set +a
forge script script/Deploy.s.sol:DeployBetOracle --rpc-url celo -vvv
```

### 5. Broadcast to Celo mainnet (real txs)
```bash
set -a; source .env; set +a
forge script script/Deploy.s.sol:DeployBetOracle --rpc-url celo --broadcast -vvv
```

Save the output:
- Factory address
- Agent Wallet address
- Prediction Contract address
- Agent ID (bytes32)

### 6. Verify contracts on CeloScan (Etherscan API v2)
```bash
set -a; source .env; set +a

# Factory
forge verify-contract <FACTORY_ADDRESS> src/BetOracleFactory.sol:BetOracleFactory \
  --verifier etherscan \
  --verifier-url "https://api.etherscan.io/v2/api?chainid=42220" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch

# Agent Wallet
forge verify-contract <AGENT_WALLET_ADDRESS> src/BetOracleAgentWallet.sol:BetOracleAgentWallet \
  --verifier etherscan \
  --verifier-url "https://api.etherscan.io/v2/api?chainid=42220" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch

# Prediction Contract
forge verify-contract <PREDICTION_CONTRACT_ADDRESS> src/BetOraclePrediction.sol:BetOraclePrediction \
  --verifier etherscan \
  --verifier-url "https://api.etherscan.io/v2/api?chainid=42220" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch
```

### 7. Authorize backend address on Agent Wallet
Add to `.env`:
```bash
AGENT_WALLET=<agent-wallet-address>
```

Then run:
```bash
set -a; source .env; set +a
forge script script/AuthorizeBackend.s.sol:AuthorizeBackend --rpc-url celo --broadcast -vvv
```

### 8. Update Railway backend env vars
```bash
BLOCKCHAIN_ENABLED=true
AGENT_WALLET=<agent-wallet-address>
PREDICTION_CONTRACT=<prediction-contract-address>
CELO_RPC_URL=https://forno.celo.org
AGENT_PRIVATE_KEY=<backend signer key>
```

### 9. Verify end-to-end
```bash
curl -X POST https://<railway-backend-url>/api/predict \
  -H "Content-Type: application/json" \
  -d '{"homeTeam":"Arsenal","awayTeam":"Chelsea","league":"EPL"}'
# Expect response with blockchain.submitted=true and txHash
```

### 10. Optional: clean build artifacts
```bash
forge clean
forge build
```

---

## Deployment result: ✅ success (Celo mainnet)
### Contracts deployed & verified on Celo mainnet (chainId 42220). Save these values:

- Deployer (EOA): 0x3E7dfBF99f10402E860Df4e7420217EF56e94cc1
- Factory: 0x430e1Bd7a0927f731E144C42feDFB12e23465cE9
- Agent Wallet: 0x8929c7C546aF792E044326ff492439F02fD13373
- Prediction Contract: 0xd5049F6550aefC772ABDa57013fB01aB718054Ef
- Agent ID (bytes32): 0x565fae5389dd06d09b03a5a1464cf02e94dd3f4e612eaf7b83a04fee42f582a5
- Broadcast TX hashes (for reference):

  - 0xc054f123a777d12eebcfbdfffa6055809424dbbcbe725bddbf9f0725ccbec6df
  - 0x1ddd471df26cee719a212afe779d4ade92718b2b03857eced41d62c5c0a2e26f
  - 0xf4e0fa8405b4bc8f4f3149e6caa3c67309f60f17459d4f0556e304bac7c9a7b0