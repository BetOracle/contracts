// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/BetOracleAgentWallet.sol";

contract AuthorizeBackend is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address agentWallet = vm.envAddress("AGENT_WALLET");
        address backendAddress = vm.envAddress("BACKEND_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);
        BetOracleAgentWallet(payable(agentWallet)).authorizeBackend(backendAddress, true);
        vm.stopBroadcast();

        console.log("Authorized backend:", backendAddress);
        console.log("On agent wallet:", agentWallet);
    }
}
