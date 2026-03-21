// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IGDAv1Forwarder {
    function createPool(
        address token,
        address admin,
        PoolConfig calldata config
    ) external returns (bool, address);

    function updateMemberUnits(
        address pool,
        address memberAddress,
        uint128 newUnits,
        bytes calldata userData
    ) external returns (bool);

    struct PoolConfig {
        bool transferabilityForUnitsOwner;
        bool distributionFromAnyAddress;
    }
}

/// @title GraffitiPool — repo.box contribution rewards via Superfluid GDA
/// @notice Push a signed commit to the wall repo, get scored, claim SUP stream
/// @dev Mergeooor signs attestations scoped to (contributor, commitHash, units, nonce).
///      Only the contributor can claim. Each attestation is single-use.
contract GraffitiPool {
    address public immutable trustedMerger;
    address public immutable pool;
    address public immutable token;
    IGDAv1Forwarder constant GDA = IGDAv1Forwarder(0x6DA13Bde224A05a288748d857b9e7DDEffd1dE08);

    mapping(bytes32 => bool) public claimed;
    uint128 public totalMembers;

    event Claimed(address indexed contributor, bytes32 commitHash, uint128 units);

    constructor(address _token, address _trustedMerger) {
        trustedMerger = _trustedMerger;
        token = _token;

        (, address _pool) = GDA.createPool(
            _token,
            address(this),
            IGDAv1Forwarder.PoolConfig(false, true) // distributionFromAnyAddress = true
        );
        pool = _pool;
    }

    function claim(
        bytes32 commitHash,
        uint128 units,
        uint256 nonce,
        bytes calldata signature
    ) external {
        require(units > 0 && units <= 10, "units 1-10");

        bytes32 digest = keccak256(abi.encodePacked(
            msg.sender, commitHash, units, nonce
        ));
        require(!claimed[digest], "already claimed");

        (bytes32 r, bytes32 s, uint8 v) = _splitSig(signature);
        bytes32 ethHash = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32", digest
        ));
        require(ecrecover(ethHash, v, r, s) == trustedMerger, "bad sig");

        claimed[digest] = true;
        totalMembers++;

        GDA.updateMemberUnits(pool, msg.sender, units, "");

        emit Claimed(msg.sender, commitHash, units);
    }

    function _splitSig(bytes calldata sig) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "bad sig len");
        r = bytes32(sig[0:32]);
        s = bytes32(sig[32:64]);
        v = uint8(sig[64]);
    }
}
