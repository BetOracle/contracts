// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./BetOracleAgentWallet.sol";
import "./BetOraclePrediction.sol";

/**
 * @title BetOracleFactory
 * @author Pope (BetOracle Team)
 * @notice Factory for deploying BetOracle agent and prediction contracts
 * @dev One-stop deployment for Celo hackathon
 */

contract BetOracleFactory {
    // ============ Events ============
    
    event AgentDeployed(
        bytes32 indexed agentId,
        address indexed agentWallet,
        string name,
        address indexed owner,
        uint256 timestamp
    );
    
    event PredictionContractDeployed(
        address indexed predictionContract,
        address indexed agentWallet,
        uint256 timestamp
    );
    
    event FullDeployment(
        bytes32 indexed agentId,
        address indexed agentWallet,
        address indexed predictionContract,
        address owner,
        uint256 timestamp
    );
    
    // ============ State ============
    
    // All deployed agents
    address[] public agentWallets;
    mapping(bytes32 => address) public agentById;
    
    // All prediction contracts
    address[] public predictionContracts;
    
    // ============ Deployment Functions ============
    
    /**
     * @notice Deploy agent wallet only (external)
     */
    function deployAgentWallet(
        bytes32 _agentId,
        string calldata _name,
        string calldata _metadataURI
    ) external returns (address agentWallet) {
        return _deployAgentWalletInternal(_agentId, _name, _metadataURI);
    }
    
    /**
     * @notice Internal deployment of agent wallet
     */
    function _deployAgentWalletInternal(
        bytes32 _agentId,
        string memory _name,
        string memory _metadataURI
    ) internal returns (address agentWallet) {
        require(agentById[_agentId] == address(0), "Agent ID already exists");
        
        agentWallet = address(new BetOracleAgentWallet(
            _agentId,
            _name,
            _metadataURI,
            msg.sender
        ));
        
        agentWallets.push(agentWallet);
        agentById[_agentId] = agentWallet;
        
        emit AgentDeployed(_agentId, agentWallet, _name, msg.sender, block.timestamp);
        
        return agentWallet;
    }
    
    /**
     * @notice Deploy prediction contract only (external)
     */
    function deployPredictionContract(
        address _agentWallet
    ) external returns (address predictionContract) {
        return _deployPredictionContractInternal(_agentWallet);
    }
    
    /**
     * @notice Internal deployment of prediction contract
     */
    function _deployPredictionContractInternal(
        address _agentWallet
    ) internal returns (address predictionContract) {
        predictionContract = address(new BetOraclePrediction());
        
        predictionContracts.push(predictionContract);
        
        // Set agent wallet on prediction contract
        BetOraclePrediction(predictionContract).setAgentWallet(_agentWallet);
        
        // Link prediction contract back onto the agent wallet.
        // This will revert if msg.sender is not the agent wallet owner,
        // making the failure immediately visible instead of silently ignored.
        BetOracleAgentWallet(payable(_agentWallet)).setPredictionContract(predictionContract);
        
        emit PredictionContractDeployed(predictionContract, _agentWallet, block.timestamp);
        
        return predictionContract;
    }
    
    /**
     * @notice Full deployment: agent + prediction contract
     * @dev Caller must manually link agent wallet to prediction contract afterwards
     */
    function deployFull(
        bytes32 _agentId,
        string calldata _name,
        string calldata _metadataURI
    ) external returns (address agentWallet, address predictionContract) {
        // Deploy agent
        agentWallet = _deployAgentWalletInternal(_agentId, _name, _metadataURI);
        
        // Deploy prediction contract
        predictionContract = _deployPredictionContractInternal(agentWallet);
        
        // Note: Caller (owner of agent wallet) must call setPredictionContract() 
        // on agent wallet to complete the linkage
        
        emit FullDeployment(
            _agentId,
            agentWallet,
            predictionContract,
            msg.sender,
            block.timestamp
        );
        
        return (agentWallet, predictionContract);
    }
    
    // ============ View Functions ============
    
    function getAgentWallet(bytes32 _agentId) external view returns (address) {
        return agentById[_agentId];
    }
    
    function getAllAgentWallets() external view returns (address[] memory) {
        return agentWallets;
    }
    
    function getAllPredictionContracts() external view returns (address[] memory) {
        return predictionContracts;
    }
    
    function getDeploymentCount() external view returns (uint256 agents, uint256 predictions) {
        return (agentWallets.length, predictionContracts.length);
    }
}
