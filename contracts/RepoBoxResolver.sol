// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title RepoBoxResolver — Hybrid onchain/offchain ENS resolver for repo.box
/// @notice Checks on-chain NFT names first, falls back to CCIP-Read for free aliases
/// @dev Implements ERC-3668 (CCIP-Read) with on-chain priority
contract RepoBoxResolver {

    /// @dev ERC-3668 CCIP-Read error
    error OffchainLookup(
        address sender,
        string[] urls,
        bytes callData,
        bytes4 callbackFunction,
        bytes extraData
    );

    error InvalidSignature();

    /// @dev Interface IDs
    bytes4 constant private ADDR_ID = 0x3b3b57de;
    bytes4 constant private RESOLVE_ID = 0x9061b923;
    bytes4 constant private ERC165_ID = 0x01ffc9a7;

    /// @dev On-chain name registry (RepoBoxNames NFT)
    IRepoBoxNames public immutable names;

    /// @dev CCIP gateway URL for free aliases
    string[] private urls;

    /// @dev Address that signs gateway responses
    address public immutable signer;

    constructor(address _names, string memory _gatewayUrl, address _signer) {
        names = IRepoBoxNames(_names);
        urls = new string[](1);
        urls[0] = _gatewayUrl;
        signer = _signer;
    }

    /// @notice Resolve a repo.box subname
    /// @dev First checks on-chain (paid NFT names), then CCIP-Read (free aliases)
    function resolve(bytes calldata name, bytes calldata data) external view returns (bytes memory) {
        // Parse subdomain from DNS-encoded name
        string memory subdomain = _parseDNSName(name);

        // Check on-chain first (paid names)
        bytes32 nameHash = keccak256(bytes(subdomain));
        uint256 tokenId = names.nameToToken(nameHash);

        if (tokenId != 0) {
            // Found on-chain — return resolved address directly
            address resolved = names.resolvedAddress(tokenId);
            return abi.encode(resolved);
        }

        // Not found on-chain — fall back to CCIP-Read for free aliases
        revert OffchainLookup(
            address(this),
            urls,
            abi.encode(name, data),
            this.resolveWithProof.selector,
            abi.encode(name, data)
        );
    }

    /// @notice CCIP-Read callback — verify gateway signature and return result
    function resolveWithProof(bytes calldata response, bytes calldata extraData)
        external
        view
        returns (bytes memory)
    {
        (bytes memory result, bytes memory signature) = abi.decode(response, (bytes, bytes));
        (bytes memory name, bytes memory data) = abi.decode(extraData, (bytes, bytes));

        // Verify gateway signature
        bytes32 messageHash = keccak256(abi.encodePacked(name, data, result));
        bytes32 ethHash = keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash));

        if (_recover(ethHash, signature) != signer) revert InvalidSignature();

        return result;
    }

    /// @notice Direct addr() lookup (also hybrid)
    function addr(bytes32 node) external view returns (address) {
        // For direct addr calls, we can't parse the name from the node hash
        // so we always go to CCIP-Read (the gateway can handle both)
        bytes memory data = abi.encodeWithSelector(this.addr.selector, node);

        revert OffchainLookup(
            address(this),
            urls,
            abi.encode(abi.encode(node), data),
            this.resolveWithProof.selector,
            abi.encode(abi.encode(node), data)
        );
    }

    function supportsInterface(bytes4 interfaceId) external pure returns (bool) {
        return interfaceId == ADDR_ID ||
               interfaceId == RESOLVE_ID ||
               interfaceId == ERC165_ID;
    }

    /// @dev Parse first label from DNS-encoded name (e.g. "\x05alice\x04repo\x03box\x00" → "alice")
    function _parseDNSName(bytes calldata name) private pure returns (string memory) {
        if (name.length == 0) return "";
        uint8 labelLen = uint8(name[0]);
        if (labelLen == 0 || labelLen + 1 > name.length) return "";
        return string(name[1:1 + labelLen]);
    }

    /// @dev ECDSA recover
    function _recover(bytes32 hash, bytes memory sig) private pure returns (address) {
        if (sig.length != 65) return address(0);
        bytes32 r; bytes32 s; uint8 v;
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        if (v < 27) v += 27;
        if (v != 27 && v != 28) return address(0);
        return ecrecover(hash, v, r, s);
    }
}

interface IRepoBoxNames {
    function nameToToken(bytes32 nameHash) external view returns (uint256);
    function resolvedAddress(uint256 tokenId) external view returns (address);
}
