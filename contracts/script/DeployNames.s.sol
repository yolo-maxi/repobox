// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console} from "forge-std/Script.sol";
import {RepoBoxNames} from "../RepoBoxNames.sol";
import {RepoBoxMint} from "../RepoBoxMint.sol";

/// @notice Deploys RepoBoxNames + RepoBoxMint in 2 txs, no extra calls.
/// Uses CREATE2 to predict Mint address before Names is deployed.
///
/// Usage:
///   OWNER=0xf8a025B42B07db05638FE596cce339707ec3cC71 \
///   RECIPIENT=0xF053A15C36f1FbCC2A281095e6f1507ea1EFc931 \
///   forge script script/DeployNames.s.sol --broadcast --rpc-url mainnet
contract DeployNames is Script {
    bytes32 constant SALT = bytes32(uint256(0xb0b));

    function run() external {
        address owner = vm.envAddress("OWNER");
        address payable recipient = payable(vm.envAddress("RECIPIENT"));

        uint64 nonce = vm.getNonce(msg.sender);

        // Predict Names address (regular CREATE at current nonce)
        address predictedNames = vm.computeCreateAddress(msg.sender, nonce);

        // Predict Mint address (CREATE2 with known salt + bytecode including predictedNames)
        bytes32 mintInitHash = keccak256(abi.encodePacked(
            type(RepoBoxMint).creationCode,
            abi.encode(predictedNames, recipient)
        ));
        address predictedMint = vm.computeCreate2Address(SALT, mintInitHash);

        console.log("Predicted Names:", predictedNames);
        console.log("Predicted Mint:", predictedMint);

        vm.startBroadcast();

        // Deploy Names with predicted Mint address baked into constructor
        RepoBoxNames names = new RepoBoxNames(owner, predictedMint);
        require(address(names) == predictedNames, "Names address mismatch");

        // Deploy Mint via CREATE2 — lands at predicted address
        RepoBoxMint mint = new RepoBoxMint{salt: SALT}(address(names), recipient);
        require(address(mint) == predictedMint, "Mint address mismatch");

        vm.stopBroadcast();

        console.log("=== DEPLOYED ===");
        console.log("RepoBoxNames:", address(names));
        console.log("RepoBoxMint:", address(mint));
        console.log("Owner:", owner);
        console.log("Recipient:", recipient);
    }
}
