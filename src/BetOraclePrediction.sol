// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title BetOraclePrediction
 * @author Pope (BetOracle Team)
 * @notice On-chain prediction registry for BetOracle agent
 * @dev Celo hackathon - Agentic prediction marketplace
 */

contract BetOraclePrediction {
    // ============ Structs ============
    
    struct Prediction {
        bytes32 predictionId;      // Unique ID for this prediction
        bytes32 matchId;           // Match identifier (league-home-away-date)
        address agent;             // Agent wallet that made prediction
        string homeTeam;           // Home team name
        string awayTeam;           // Away team name
        string league;             // League code (EPL, LaLiga, etc.)
        uint8 prediction;        // 0=HOME_WIN, 1=DRAW, 2=AWAY_WIN
        uint256 confidence;        // Confidence score (0-10000 = 0-100%)
        uint256 timestamp;         // When prediction was made
        uint256 matchDate;         // When match occurs
        bool resolved;             // Has match been resolved
        uint8 outcome;             // Actual outcome (0=HOME_WIN, 1=DRAW, 2=AWAY_WIN, 255=UNRESOLVED)
        uint256 stakeAmount;       // Amount staked on prediction (for paid predictions)
    }

    // ============ State Variables ============
    
    // Agent wallet address (ERC-8004 compliant)
    address public agentWallet;
    
    // Contract owner (for admin functions)
    address public owner;
    
    // Prediction storage: predictionId => Prediction
    mapping(bytes32 => Prediction) public predictions;
    
    // All prediction IDs for querying
    bytes32[] public predictionIds;
    
    // Agent stats
    uint256 public totalPredictions;
    uint256 public correctPredictions;
    uint256 public resolvedPredictions;   // How many have been resolved (denominator for accuracy)
    uint256 public totalStaked;
    
    // ============ Events ============
    
    event PredictionSubmitted(
        bytes32 indexed predictionId,
        bytes32 indexed matchId,
        address indexed agent,
        string homeTeam,
        string awayTeam,
        uint8 prediction,
        uint256 confidence,
        uint256 timestamp
    );
    
    event PredictionResolved(
        bytes32 indexed predictionId,
        bytes32 indexed matchId,
        uint8 outcome,
        bool correct,
        uint256 timestamp
    );
    
    event AgentWalletUpdated(
        address indexed oldWallet,
        address indexed newWallet
    );

    // ============ Modifiers ============
    
    modifier onlyAgent() {
        require(msg.sender == agentWallet, "Only agent wallet can call");
        _;
    }
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call");
        _;
    }
    
    // ============ Constructor ============
    
    constructor() {
        owner = msg.sender;
        // Agent wallet set separately after ERC-8004 registration
    }
    
    // ============ Admin Functions ============
    
    /**
     * @notice Set the agent wallet address (after ERC-8004 registration)
     * @param _agentWallet The registered agent wallet address
     */
    function setAgentWallet(address _agentWallet) external onlyOwner {
        require(_agentWallet != address(0), "Invalid agent wallet");
        address oldWallet = agentWallet;
        agentWallet = _agentWallet;
        emit AgentWalletUpdated(oldWallet, _agentWallet);
    }
    
    /**
     * @notice Submit a new prediction (called by agent)
     * @param _predictionId Unique ID for this prediction
     * @param _matchId Match identifier
     * @param _homeTeam Home team name
     * @param _awayTeam Away team name
     * @param _league League code
     * @param _prediction 0=HOME_WIN, 1=DRAW, 2=AWAY_WIN
     * @param _confidence Confidence score (0-10000)
     * @param _matchDate When match occurs
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
    ) external onlyAgent returns (bool) {
        require(_prediction <= 2, "Invalid prediction (0-2)");
        require(_confidence <= 10000, "Confidence must be 0-10000");
        require(predictions[_predictionId].timestamp == 0, "Prediction already exists");
        // Allow matchDate == 0 (backend may not always know kick-off time).
        // If provided, it must be at least 1 hour from now to prevent resolving
        // before the match has started.
        if (_matchDate > 0) {
            require(_matchDate > block.timestamp, "Match must be in future");
        }
        
        _storePrediction(_predictionId, _matchId, _homeTeam, _awayTeam, _league, _prediction, _confidence, _matchDate, 0);
        
        return true;
    }
    
    /**
     * @notice Internal function to store prediction data
     */
    function _storePrediction(
        bytes32 _predictionId,
        bytes32 _matchId,
        string calldata _homeTeam,
        string calldata _awayTeam,
        string calldata _league,
        uint8 _prediction,
        uint256 _confidence,
        uint256 _matchDate,
        uint256 _stakeAmount
    ) internal {
        predictions[_predictionId] = Prediction({
            predictionId: _predictionId,
            matchId: _matchId,
            agent: agentWallet,
            homeTeam: _homeTeam,
            awayTeam: _awayTeam,
            league: _league,
            prediction: _prediction,
            confidence: _confidence,
            timestamp: block.timestamp,
            matchDate: _matchDate,
            resolved: false,
            outcome: 255, // UNRESOLVED
            stakeAmount: _stakeAmount
        });
        
        predictionIds.push(_predictionId);
        totalPredictions++;
        
        emit PredictionSubmitted(
            _predictionId,
            _matchId,
            agentWallet,
            _homeTeam,
            _awayTeam,
            _prediction,
            _confidence,
            block.timestamp
        );
    }
    
    /**
     * @notice Submit prediction with stake (payable)
     */
    function submitPredictionWithStake(
        bytes32 _predictionId,
        bytes32 _matchId,
        string calldata _homeTeam,
        string calldata _awayTeam,
        string calldata _league,
        uint8 _prediction,
        uint256 _confidence,
        uint256 _matchDate
    ) external payable onlyAgent returns (bool) {
        require(msg.value > 0, "Must stake some amount");
        require(_prediction <= 2, "Invalid prediction (0-2)");
        require(_confidence <= 10000, "Confidence must be 0-10000");
        require(predictions[_predictionId].timestamp == 0, "Prediction already exists");
        require(_matchDate > block.timestamp, "Match must be in future");
        
        _storePrediction(_predictionId, _matchId, _homeTeam, _awayTeam, _league, _prediction, _confidence, _matchDate, msg.value);
        
        totalStaked += msg.value;
        
        return true;
    }
    
    /**
     * @notice Resolve a prediction with actual outcome
     * @param _predictionId The prediction to resolve
     * @param _outcome Actual outcome (0=HOME_WIN, 1=DRAW, 2=AWAY_WIN)
     */
    function resolvePrediction(
        bytes32 _predictionId,
        uint8 _outcome
    ) external onlyAgent {
        require(_outcome <= 2, "Invalid outcome (0-2)");
        
        Prediction storage pred = predictions[_predictionId];
        require(pred.timestamp > 0, "Prediction not found");
        require(!pred.resolved, "Already resolved");
        // If matchDate was provided, enforce the match has started (with 1h buffer).
        if (pred.matchDate > 0) {
            require(
                block.timestamp >= pred.matchDate - 1 hours,
                "Match has not started yet"
            );
        }
        
        pred.resolved = true;
        pred.outcome = _outcome;
        
        resolvedPredictions++;
        bool correct = (pred.prediction == _outcome);
        if (correct) {
            correctPredictions++;
        }
        
        emit PredictionResolved(
            _predictionId,
            pred.matchId,
            _outcome,
            correct,
            block.timestamp
        );
    }
    
    // ============ View Functions ============
    
    /**
     * @notice Get prediction by ID
     */
    function getPrediction(bytes32 _predictionId) external view returns (Prediction memory) {
        return predictions[_predictionId];
    }
    
    /**
     * @notice Get all predictions (paginated)
     */
    function getPredictions(uint256 _offset, uint256 _limit) 
        external 
        view 
        returns (Prediction[] memory) 
    {
        uint256 end = _offset + _limit;
        if (end > predictionIds.length) {
            end = predictionIds.length;
        }
        
        Prediction[] memory result = new Prediction[](end - _offset);
        for (uint256 i = _offset; i < end; i++) {
            result[i - _offset] = predictions[predictionIds[i]];
        }
        
        return result;
    }
    
    /**
     * @notice Get agent accuracy over resolved predictions only
     * @return Accuracy as percentage * 100 (e.g. 6750 = 67.50%)
     */
    function getAgentAccuracy() external view returns (uint256) {
        if (resolvedPredictions == 0) return 0;
        return (correctPredictions * 10000) / resolvedPredictions;
    }
    
    /**
     * @notice Check if prediction is correct (after resolution)
     */
    function isPredictionCorrect(bytes32 _predictionId) external view returns (bool) {
        Prediction memory pred = predictions[_predictionId];
        require(pred.resolved, "Not resolved yet");
        return pred.prediction == pred.outcome;
    }
    
    // ============ Utility Functions ============
    
    /**
     * @notice Generate match ID from components
     */
    function generateMatchId(
        string calldata _league,
        string calldata _homeTeam,
        string calldata _awayTeam,
        uint256 _date
    ) external pure returns (bytes32) {
        return keccak256(abi.encodePacked(_league, _homeTeam, _awayTeam, _date));
    }
    
    /**
     * @notice Generate prediction ID
     */
    function generatePredictionId(
        bytes32 _matchId,
        uint256 _timestamp
    ) external pure returns (bytes32) {
        return keccak256(abi.encodePacked(_matchId, _timestamp));
    }
}
