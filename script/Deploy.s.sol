// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/BetOracleFactory.sol";
import "../src/BetOracleAgentWallet.sol";
import "../src/BetOraclePrediction.sol";

/**
 * @title DeployBetOracle
 * @notice Deployment script for BetOracle contracts
 * @dev Run with: forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast
 */
contract DeployBetOracle is Script {
    function run() external {
        // Get deployment private key from environment
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        // Agent configuration (can also come from env)
        bytes32 agentId = keccak256(abi.encodePacked("BetOracle-Agent-001"));
        string memory agentName = "BetOracle Prediction Agent";
        string memory metadataURI = "https://betoracle.xyz/agent-metadata.json";

        vm.startBroadcast(deployerPrivateKey);

        console.log("Starting BetOracle deployment...");
        console.log("Deployer:", vm.addr(deployerPrivateKey));

        // Deploy factory
        BetOracleFactory factory = new BetOracleFactory();
        console.log("Factory deployed at:", address(factory));

        // Deploy full stack: agent wallet + prediction contract
        (address agentWallet, address predictionContract) = factory.deployFull(agentId, agentName, metadataURI);

        // Complete linkage: the agent wallet owner (deployer) must set the prediction contract
        BetOracleAgentWallet(payable(agentWallet)).setPredictionContract(predictionContract);

        console.log("Agent Wallet deployed at:", agentWallet);
        console.log("Prediction Contract deployed at:", predictionContract);
        console.log("Agent ID:", uint256(agentId));

        // Optional: Authorize backend to submit predictions
        // address backend = vm.envAddress("BACKEND_ADDRESS");
        // BetOracleAgentWallet(payable(agentWallet)).authorizeBackend(backend, true);

        vm.stopBroadcast();

        // Log deployment info
        console.log("\n=== DEPLOYMENT COMPLETE ===");
        console.log("Agent ID (save this for .env):");
        console.logBytes32(agentId);
        console.log("Agent Wallet:", agentWallet);
        console.log("Prediction Contract:", predictionContract);
        console.log("Factory:", address(factory));
    }
}

/**
 * @title DeployAgentOnly
 * @notice Deploy just the agent wallet
 */
contract DeployAgentOnly is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        bytes32 agentId = keccak256(abi.encodePacked("BetOracle-Agent-002"));
        string memory agentName = "BetOracle Agent";
        string memory metadataURI = "https://betoracle.xyz/agent.json";

        vm.startBroadcast(deployerPrivateKey);

        BetOracleAgentWallet agent =
            new BetOracleAgentWallet(agentId, agentName, metadataURI, vm.addr(deployerPrivateKey));

        vm.stopBroadcast();

        console.log("Agent Wallet deployed at:", address(agent));
        console.log("Agent ID:");
        console.logBytes32(agentId);
    }
}
