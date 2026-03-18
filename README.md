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

