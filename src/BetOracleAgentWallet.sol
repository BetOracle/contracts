// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title BetOracleAgentWallet
 * @author Pope (BetOracle Team)
 * @notice ERC-8004 compliant agent wallet for BetOracle
 * @dev Simplified agent wallet for Celo hackathon
 * 
 * ERC-8004: Agent Wallet Standard
 * - Holds funds for agent operations
 * - Signs/executes transactions on behalf of agent
 * - Records reputation via on-chain predictions
 */

contract BetOracleAgentWallet is Ownable {
    // ============ Structs ============
    
    struct AgentProfile {
        bytes32 agentId;           // Unique agent identifier
        string name;               // Agent name
        string metadataURI;        // IPFS/Arweave link to agent metadata
        uint256 createdAt;         // Registration timestamp
        bool active;               // Is agent active
    }
    
    struct Reputation {
        uint256 totalPredictions;
        uint256 correctPredictions;
        uint256 totalStaked;
        uint256 reputationScore;   // 0-10000 (0-100% with 2 decimals)
        uint256 lastUpdated;
    }
    
    // ============ State Variables ============
    
    AgentProfile public profile;
    Reputation public reputation;
    
    // Prediction contract this agent interacts with
    address public predictionContract;
    
    // Authorized backends that can trigger agent actions
    mapping(address => bool) public authorizedBackends;
    
    // x402 payment tracking (simplified)
    mapping(bytes32 => bool) public processedPayments;
    
    // ============ Events ============
    
    event AgentRegistered(
        bytes32 indexed agentId,
        string name,
        address indexed owner,
        uint256 timestamp
    );
    
    event BackendAuthorized(
        address indexed backend,
        bool authorized
    );
    
    event PredictionSubmitted(
        bytes32 indexed predictionId,
        bytes32 indexed matchId,
        uint256 confidence,
        uint256 timestamp
    );
    
    event ReputationUpdated(
        uint256 newScore,
        uint256 totalPredictions,
        uint256 correctPredictions,
        uint256 timestamp
    );
    
    event PaymentProcessed(
        bytes32 indexed paymentId,
        address indexed token,
        uint256 amount,
        uint256 timestamp
    );
    
    // ============ Modifiers ============
    
    modifier onlyAuthorized() {
        require(
            msg.sender == owner() || authorizedBackends[msg.sender],
            "Not authorized"
        );
        _;
    }
    
    // ============ Constructor ============
    
    constructor(
        bytes32 _agentId,
        string memory _name,
        string memory _metadataURI,
        address _owner
    ) Ownable(_owner) {
        require(_agentId != bytes32(0), "Invalid agent ID");
        require(bytes(_name).length > 0, "Name required");
        
        profile = AgentProfile({
            agentId: _agentId,
            name: _name,
            metadataURI: _metadataURI,
            createdAt: block.timestamp,
            active: true
        });
        
        emit AgentRegistered(_agentId, _name, _owner, block.timestamp);
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Authorize a backend to act on behalf of agent
     */
    function authorizeBackend(address _backend, bool _authorized) external onlyOwner {
        authorizedBackends[_backend] = _authorized;
        emit BackendAuthorized(_backend, _authorized);
    }
    
    /**
     * @notice Set prediction contract address
     */
    function setPredictionContract(address _contract) external onlyOwner {
        require(_contract != address(0), "Invalid contract");
        predictionContract = _contract;
    }
    
    /**
     * @notice Update agent metadata
     */
    function updateMetadata(string calldata _metadataURI) external onlyOwner {
        profile.metadataURI = _metadataURI;
    }
    
    /**
     * @notice Activate/deactivate agent
     */
    function setActive(bool _active) external onlyOwner {
        profile.active = _active;
    }
    
    // ============ Core Agent Functions ============
    
    /**
     * @notice Submit prediction through agent wallet
     * @dev Called by authorized backend (your Python backend)
     */
    function submitPrediction(
        bytes32 _predictionId,
        bytes32 _matchId,
        string calldata _homeTeam,
        string calldata _awayTeam,
        string calldata _league,
        uint8 _prediction,
        uint256 _confidence,
        uint256 _matchDate
    ) external onlyAuthorized returns (bool) {
        require(predictionContract != address(0), "Prediction contract not set");
        require(profile.active, "Agent not active");
        
        // Call prediction contract
        (bool success, ) = predictionContract.call(
            abi.encodeWithSelector(
                bytes4(keccak256("submitPrediction(bytes32,bytes32,string,string,string,uint8,uint256,uint256)")),
                _predictionId,
                _matchId,
                _homeTeam,
                _awayTeam,
                _league,
                _prediction,
                _confidence,
                _matchDate
            )
        );
        
        if (success) {
            reputation.totalPredictions++;
            reputation.lastUpdated = block.timestamp;
            
            emit PredictionSubmitted(_predictionId, _matchId, _confidence, block.timestamp);
        }
        
        return success;
    }
    
    /**
     * @notice Update reputation after prediction resolution
     * @dev Called by prediction contract or owner
     */
    function updateReputation(
        uint256 _totalPredictions,
        uint256 _correctPredictions
    ) external onlyOwner {
        reputation.totalPredictions = _totalPredictions;
        reputation.correctPredictions = _correctPredictions;
        
        // Calculate reputation score (0-10000)
        if (_totalPredictions > 0) {
            reputation.reputationScore = (_correctPredictions * 10000) / _totalPredictions;
        }
        
        reputation.lastUpdated = block.timestamp;
        
        emit ReputationUpdated(
            reputation.reputationScore,
            _totalPredictions,
            _correctPredictions,
            block.timestamp
        );
    }
    
    // ============ Payment Functions (x402 simplified) ============
    
    /**
     * @notice Process x402 payment to a recipient
     * @dev Authorization is enforced by onlyAuthorized — only the owner or
     *      an explicitly whitelisted backend can call this. No separate signature
     *      is needed because the caller is already authenticated on-chain.
     */
    function processPayment(
        bytes32 _paymentId,
        address _token,
        uint256 _amount,
        address _recipient
    ) external onlyAuthorized returns (bool) {
        require(!processedPayments[_paymentId], "Payment already processed");
        require(_recipient != address(0), "Invalid recipient");
        require(_amount > 0, "Amount must be > 0");
        
        // Mark as processed first (re-entrancy guard)
        processedPayments[_paymentId] = true;
        
        // Transfer tokens
        if (_token == address(0)) {
            // Native token (CELO)
            require(address(this).balance >= _amount, "Insufficient CELO balance");
            (bool success, ) = _recipient.call{value: _amount}("");
            require(success, "CELO transfer failed");
        } else {
            // ERC20 token
            IERC20 token = IERC20(_token);
            require(token.transfer(_recipient, _amount), "Token transfer failed");
        }
        
        emit PaymentProcessed(_paymentId, _token, _amount, block.timestamp);
        
        return true;
    }
    
    /**
     * @notice Check if payment was processed
     */
    function isPaymentProcessed(bytes32 _paymentId) external view returns (bool) {
        return processedPayments[_paymentId];
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get full agent profile
     */
    function getProfile() external view returns (AgentProfile memory) {
        return profile;
    }
    
    /**
     * @notice Get reputation stats
     */
    function getReputation() external view returns (Reputation memory) {
        return reputation;
    }
    
    /**
     * @notice Check if address is authorized backend
     */
    function isAuthorized(address _addr) external view returns (bool) {
        return authorizedBackends[_addr] || _addr == owner();
    }
    
    /**
     * @notice Get wallet balance
     */
    function getBalance(address _token) external view returns (uint256) {
        if (_token == address(0)) {
            return address(this).balance;
        }
        return IERC20(_token).balanceOf(address(this));
    }
    
    // ============ Fallback ============
    
    receive() external payable {}
    fallback() external payable {}
}
