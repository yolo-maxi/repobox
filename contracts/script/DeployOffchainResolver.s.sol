// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../OffchainResolver.sol";

/// @title DeployOffchainResolver - Deployment script for OffchainResolver
/// @notice Deploys the OffchainResolver contract for repo.box ENS resolution
contract DeployOffchainResolver {

    /// @notice Deploy the OffchainResolver contract
    /// @param gatewayUrl The gateway URL for CCIP requests (e.g., "https://ens.repo.box/{sender}/{data}.json")
    /// @param signer The address authorized to sign gateway responses
    /// @return resolver The deployed OffchainResolver contract instance
    function deploy(string memory gatewayUrl, address signer) public returns (OffchainResolver) {
        // Deploy the resolver
        OffchainResolver resolver = new OffchainResolver(gatewayUrl, signer);

        return resolver;
    }

    /// @notice Get the calldata for setting this resolver on repo.box
    /// @param resolverAddress The address of the deployed resolver
    /// @return calldata The encoded setResolver call for the ENS registry
    function getSetResolverCalldata(address resolverAddress) public pure returns (bytes memory) {
        // ENS Registry setResolver function selector: 0x1896f70a
        // For repo.box domain node hash
        bytes32 repoBoxNode = 0x3c4c0b2bfab38b2e9d7c50b19dcd24c1ee3a1b0b6d4e2b8c3f0a5e8b7d9c4a8f; // Placeholder - real hash would be computed

        return abi.encodeWithSelector(
            0x1896f70a, // setResolver(bytes32,address)
            repoBoxNode,
            resolverAddress
        );
    }
}

/// @title SimpleDeployScript - Simple deployment for testing
/// @notice Basic deployment script compatible with `forge script`
contract SimpleDeployScript {
    function run() external {
        // These would be passed as environment variables in real deployment
        string memory gatewayUrl = "https://ens.repo.box/{sender}/{data}.json";
        address signer = address(0); // Would be set to actual signer address

        // Deploy the resolver
        OffchainResolver resolver = new OffchainResolver(gatewayUrl, signer);

        // Log the deployment
        console.log("OffchainResolver deployed at:", address(resolver));
        console.log("Gateway URL:", gatewayUrl);
        console.log("Signer address:", signer);
    }
}

// Import console for logging
library console {
    address constant CONSOLE_ADDRESS = address(0x000000000000000000636F6e736F6c652e6c6f67);

    function log(string memory p0, address p1) internal view {
        _sendLogPayload(abi.encodeWithSignature("log(string,address)", p0, p1));
    }

    function log(string memory p0, string memory p1) internal view {
        _sendLogPayload(abi.encodeWithSignature("log(string,string)", p0, p1));
    }

    function log(string memory p0) internal view {
        _sendLogPayload(abi.encodeWithSignature("log(string)", p0));
    }

    function _sendLogPayload(bytes memory payload) private view {
        uint256 payloadLength = payload.length;
        address consoleAddress = CONSOLE_ADDRESS;
        assembly {
            let payloadStart := add(payload, 32)
            let r := staticcall(gas(), consoleAddress, payloadStart, payloadLength, 0, 0)
        }
    }
}