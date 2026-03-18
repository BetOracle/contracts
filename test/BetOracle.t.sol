// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/BetOraclePrediction.sol";
import "../src/BetOracleAgentWallet.sol";
import "../src/BetOracleFactory.sol";

contract BetOracleTest is Test {
    BetOraclePrediction public predictionContract;
    BetOracleAgentWallet public agentWallet;
    BetOracleFactory public factory;

    address public owner;
    address public backend;
    address public user;

    bytes32 public agentId;

    function setUp() public {
        owner = address(this);
        backend = makeAddr("backend");
        user = makeAddr("user");

        agentId = keccak256(abi.encodePacked("Test-Agent-001"));

        // Deploy agent wallet
        agentWallet = new BetOracleAgentWallet(agentId, "Test Agent", "https://test.uri", owner);

        // Deploy prediction contract
        predictionContract = new BetOraclePrediction();

        // Set agent wallet on prediction contract
        predictionContract.setAgentWallet(address(agentWallet));

        // Set prediction contract on agent wallet
        agentWallet.setPredictionContract(address(predictionContract));

        // Authorize backend
        agentWallet.authorizeBackend(backend, true);

        // Deploy factory
        factory = new BetOracleFactory();
    }

    // ============ Agent Wallet Tests ============

    function test_AgentProfile() public view {
        (bytes32 agentId, string memory name, string memory metadataURI, uint256 createdAt, bool active) =
            agentWallet.profile();
        assertEq(agentId, agentId);
        assertEq(name, "Test Agent");
        assertEq(metadataURI, "https://test.uri");
        assertTrue(active);
        assertGt(createdAt, 0);
    }

    function test_AuthorizeBackend() public {
        address newBackend = makeAddr("new-backend");

        vm.expectEmit(true, true, false, false);
        emit BetOracleAgentWallet.BackendAuthorized(newBackend, true);

        agentWallet.authorizeBackend(newBackend, true);

        assertTrue(agentWallet.authorizedBackends(newBackend));
        assertTrue(agentWallet.isAuthorized(newBackend));
    }

    function test_SubmitPrediction() public {
        bytes32 matchId = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));

        vm.prank(backend);
        bool success = agentWallet.submitPrediction(
            predictionId,
            matchId,
            "Arsenal",
            "Chelsea",
            "EPL",
            0, // HOME_WIN
            7500, // 75% confidence
            block.timestamp + 1 days
        );

        assertTrue(success);

        // Check prediction was recorded
        BetOraclePrediction.Prediction memory pred = predictionContract.getPrediction(predictionId);
        assertEq(pred.predictionId, predictionId);
        assertEq(pred.homeTeam, "Arsenal");
        assertEq(pred.awayTeam, "Chelsea");
        assertEq(pred.prediction, 0);
        assertEq(pred.confidence, 7500);
    }

    // ============ Prediction Contract Tests ============

    function test_SubmitPrediction_Direct() public {
        bytes32 matchId = keccak256("EPL-MCI-LIV-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));

        vm.prank(address(agentWallet));
        predictionContract.submitPrediction(
            predictionId,
            matchId,
            "Man City",
            "Liverpool",
            "EPL",
            1, // DRAW
            5000,
            block.timestamp + 2 days
        );

        BetOraclePrediction.Prediction memory pred = predictionContract.getPrediction(predictionId);
        assertEq(pred.homeTeam, "Man City");
        assertEq(pred.awayTeam, "Liverpool");
        assertEq(pred.league, "EPL");
    }

    function test_ResolvePrediction() public {
        // First submit a prediction for a future match
        bytes32 matchId = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));
        uint256 matchDate = block.timestamp + 2 days;

        vm.prank(address(agentWallet));
        predictionContract.submitPrediction(
            predictionId,
            matchId,
            "Arsenal",
            "Chelsea",
            "EPL",
            0, // HOME_WIN
            8000,
            matchDate
        );

        // Warp time forward to after match date
        vm.warp(matchDate + 1 hours);

        // Resolve it
        vm.prank(address(agentWallet));
        predictionContract.resolvePrediction(predictionId, 0); // Actually home won

        BetOraclePrediction.Prediction memory pred = predictionContract.getPrediction(predictionId);
        assertTrue(pred.resolved);
        assertEq(pred.outcome, 0);
    }

    function test_GetAgentAccuracy() public {
        // Submit and resolve 2 predictions
        bytes32 matchId1 = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId1 = keccak256(abi.encodePacked(matchId1, block.timestamp));
        uint256 matchDate1 = block.timestamp + 2 days;

        vm.prank(address(agentWallet));
        predictionContract.submitPrediction(predictionId1, matchId1, "Arsenal", "Chelsea", "EPL", 0, 8000, matchDate1);

        bytes32 matchId2 = keccak256("EPL-MCI-LIV-20260316");
        bytes32 predictionId2 = keccak256(abi.encodePacked(matchId2, block.timestamp + 1));
        uint256 matchDate2 = block.timestamp + 3 days;

        vm.prank(address(agentWallet));
        predictionContract.submitPrediction(
            predictionId2, matchId2, "Man City", "Liverpool", "EPL", 0, 7000, matchDate2
        );

        // Warp to after both matches
        vm.warp(matchDate2 + 1 hours);

        // Resolve both correctly
        vm.startPrank(address(agentWallet));
        predictionContract.resolvePrediction(predictionId1, 0);
        predictionContract.resolvePrediction(predictionId2, 0);
        vm.stopPrank();

        uint256 accuracy = predictionContract.getAgentAccuracy();
        assertEq(accuracy, 10000); // 100% accuracy
    }

    // ============ Factory Tests ============

    function test_FactoryDeployFull() public {
        bytes32 newAgentId = keccak256("Factory-Agent-001");

        (address wallet, address predContract) = factory.deployFull(newAgentId, "Factory Agent", "https://factory.uri");

        assertTrue(wallet != address(0));
        assertTrue(predContract != address(0));
        assertEq(factory.agentById(newAgentId), wallet);

        // Link contracts (must be done by owner)
        BetOracleAgentWallet deployedWallet = BetOracleAgentWallet(payable(wallet));
        deployedWallet.setPredictionContract(predContract);

        // Verify contract linkage
        assertEq(deployedWallet.predictionContract(), predContract);
    }

    function test_FactoryGetAllAgents() public {
        factory.deployFull(keccak256("Agent-1"), "Agent 1", "uri1");
        factory.deployFull(keccak256("Agent-2"), "Agent 2", "uri2");
        factory.deployFull(keccak256("Agent-3"), "Agent 3", "uri3");

        address[] memory agents = factory.getAllAgentWallets();
        assertEq(agents.length, 3);
    }

    // ============ Edge Cases ============

    function test_RevertUnauthorizedPrediction() public {
        bytes32 matchId = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));

        // Try to submit from unauthorized address
        vm.prank(user);
        vm.expectRevert("Only agent wallet can call");
        predictionContract.submitPrediction(
            predictionId, matchId, "Arsenal", "Chelsea", "EPL", 0, 8000, block.timestamp + 1 days
        );
    }

    function test_RevertDuplicatePrediction() public {
        bytes32 matchId = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));

        vm.startPrank(address(agentWallet));
        predictionContract.submitPrediction(
            predictionId, matchId, "Arsenal", "Chelsea", "EPL", 0, 8000, block.timestamp + 1 days
        );

        // Try duplicate
        vm.expectRevert("Prediction already exists");
        predictionContract.submitPrediction(
            predictionId, matchId, "Arsenal", "Chelsea", "EPL", 0, 8000, block.timestamp + 1 days
        );
        vm.stopPrank();
    }

    // ============ Gas Tests ============

    function test_GasSubmitPrediction() public {
        bytes32 matchId = keccak256("EPL-ARS-CHE-20260316");
        bytes32 predictionId = keccak256(abi.encodePacked(matchId, block.timestamp));

        vm.prank(address(agentWallet));
        uint256 gasStart = gasleft();

        predictionContract.submitPrediction(
            predictionId, matchId, "Arsenal", "Chelsea", "EPL", 0, 8000, block.timestamp + 1 days
        );

        uint256 gasUsed = gasStart - gasleft();
        console.log("Gas used for submitPrediction:", gasUsed);

        // Should be reasonable on Celo (< 500k gas with via-ir)
        assertLt(gasUsed, 500000);
    }
}
