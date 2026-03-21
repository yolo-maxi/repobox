// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./lib/forge-std/src/interfaces/IERC165.sol";

/// @title OffchainResolver - CCIP-Read resolver for repo.box ENS names
/// @notice Resolves repo.box subdomain aliases to Ethereum addresses using offchain data
/// @dev Implements ERC-3668 (CCIP-Read) for offchain resolution
contract OffchainResolver is IERC165 {

    /// @dev Error to trigger CCIP-Read as per ERC-3668
    error OffchainLookup(
        address sender,
        string[] urls,
        bytes callData,
        bytes4 callbackFunction,
        bytes extraData
    );

    /// @dev Interface for ENS resolver
    bytes4 constant private ADDR_INTERFACE_ID = 0x3b3b57de;
    bytes4 constant private RESOLVE_INTERFACE_ID = 0x9061b923;

    /// @dev Gateway URL template - will be https://ens.repo.box/{sender}/{data}.json
    string[] private urls;

    /// @dev Address authorized to sign gateway responses
    address public immutable signer;

    /// @dev Events
    event SignerUpdated(address indexed newSigner);

    /// @dev Errors
    error InvalidSignature();
    error InvalidData();

    /// @param _gatewayUrl The gateway URL for CCIP requests
    /// @param _signer The address authorized to sign responses
    constructor(string memory _gatewayUrl, address _signer) {
        urls = new string[](1);
        urls[0] = _gatewayUrl;
        signer = _signer;
    }

    /// @notice Resolve an ENS name to an address (ERC-3668 CCIP-Read entry point)
    /// @param name The ENS name to resolve
    /// @param data The resolution request data
    /// @return Always reverts with OffchainLookup to trigger CCIP-Read
    function resolve(bytes calldata name, bytes calldata data)
        external
        view
        returns (bytes memory)
    {
        revert OffchainLookup(
            address(this),
            urls,
            abi.encodeWithSelector(IResolverService.resolve.selector, name, data),
            this.resolveWithProof.selector,
            abi.encode(name, data)
        );
    }

    /// @notice Callback function to verify gateway response and return result
    /// @param response The response from the gateway including signature
    /// @param extraData The original request data
    /// @return The resolved address as bytes
    function resolveWithProof(bytes calldata response, bytes calldata extraData)
        external
        view
        returns (bytes memory)
    {
        // Decode the gateway response: [result, signature]
        (bytes memory result, bytes memory signature) = abi.decode(response, (bytes, bytes));

        // Decode original request for verification
        (bytes memory name, bytes memory data) = abi.decode(extraData, (bytes, bytes));

        // Create message hash for signature verification
        bytes32 messageHash = keccak256(abi.encodePacked(name, data, result));
        bytes32 ethSignedMessageHash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash));

        // Verify signature
        address recoveredSigner = recoverSigner(ethSignedMessageHash, signature);
        if (recoveredSigner != signer) {
            revert InvalidSignature();
        }

        return result;
    }

    /// @notice Standard ENS addr() function for address resolution
    /// @param node The ENS node hash
    /// @return The resolved address
    function addr(bytes32 node) external view returns (address) {
        // Convert node to name and create resolution request
        bytes memory data = abi.encodeWithSelector(this.addr.selector, node);

        revert OffchainLookup(
            address(this),
            urls,
            abi.encodeWithSelector(IResolverService.resolve.selector, abi.encode(node), data),
            this.resolveWithProof.selector,
            abi.encode(abi.encode(node), data)
        );
    }

    /// @notice Check if this contract supports a given interface
    /// @param interfaceId The interface identifier
    /// @return True if the interface is supported
    function supportsInterface(bytes4 interfaceId) external pure override returns (bool) {
        return interfaceId == ADDR_INTERFACE_ID ||
               interfaceId == RESOLVE_INTERFACE_ID ||
               interfaceId == type(IERC165).interfaceId;
    }

    /// @dev Recover signer address from signature
    /// @param hash The message hash that was signed
    /// @param signature The signature bytes
    /// @return The address that created the signature
    function recoverSigner(bytes32 hash, bytes memory signature) private pure returns (address) {
        if (signature.length != 65) {
            revert InvalidSignature();
        }

        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }

        // EIP-2: v should be 27 or 28
        if (v < 27) {
            v += 27;
        }

        if (v != 27 && v != 28) {
            revert InvalidSignature();
        }

        return ecrecover(hash, v, r, s);
    }
}

/// @dev Interface for the gateway service
interface IResolverService {
    function resolve(bytes calldata name, bytes calldata data) external view returns (bytes memory);
}