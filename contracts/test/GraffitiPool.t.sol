// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";

// ── Inline testable version (no hardcoded addresses) ────────────────────

interface IGDAv1Forwarder {
    struct PoolConfig {
        bool transferabilityForUnitsOwner;
        bool distributionFromAnyAddress;
    }
    function createPool(address token, address admin, PoolConfig calldata config)
        external returns (bool, address);
    function updateMemberUnits(address pool, address memberAddress, uint128 newUnits, bytes calldata userData)
        external returns (bool);
}

interface IHost {
    function callAgreement(address agreementClass, bytes calldata callData, bytes calldata userData)
        external returns (bytes memory);
}

contract GraffitiPoolHarness {
    address public immutable trustedMerger;
    address public immutable pool;
    address public immutable token;
    address public gda;
    address public host;
    address public gdaV1;

    mapping(bytes32 => bool) public claimed;
    uint128 public totalMembers;

    event Claimed(address indexed contributor, bytes32 commitHash, uint128 units);

    constructor(
        address _token,
        address _trustedMerger,
        address _gda,
        address _host,
        address _gdaV1
    ) {
        trustedMerger = _trustedMerger;
        token = _token;
        gda = _gda;
        host = _host;
        gdaV1 = _gdaV1;

        (, address _pool) = IGDAv1Forwarder(_gda).createPool(
            _token,
            address(this),
            IGDAv1Forwarder.PoolConfig(false, true)
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

        IGDAv1Forwarder(gda).updateMemberUnits(pool, msg.sender, units, "");

        IHost(host).callAgreement(
            gdaV1,
            abi.encodeWithSignature(
                "tryConnectPoolFor(address,address,bytes)",
                pool, msg.sender, new bytes(0)
            ),
            new bytes(0)
        );

        emit Claimed(msg.sender, commitHash, units);
    }

    function _splitSig(bytes calldata sig) internal pure returns (bytes32 r, bytes32 s, uint8 v) {
        require(sig.length == 65, "bad sig len");
        r = bytes32(sig[0:32]);
        s = bytes32(sig[32:64]);
        v = uint8(sig[64]);
    }
}

// ── Mocks ───────────────────────────────────────────────────────────────

contract MockGDA {
    address public lastMember;
    uint128 public lastUnits;
    uint256 public updateCount;

    function createPool(address, address, IGDAv1Forwarder.PoolConfig calldata)
        external pure returns (bool, address)
    {
        return (true, address(0xBEEF));
    }

    function updateMemberUnits(address, address member, uint128 units, bytes calldata)
        external returns (bool)
    {
        lastMember = member;
        lastUnits = units;
        updateCount++;
        return true;
    }
}

contract MockHost {
    uint256 public callCount;

    function callAgreement(address, bytes calldata, bytes calldata)
        external returns (bytes memory)
    {
        callCount++;
        return "";
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

contract GraffitiPoolTest is Test {
    event Claimed(address indexed contributor, bytes32 commitHash, uint128 units);

    GraffitiPoolHarness pool;
    MockGDA mockGda;
    MockHost mockHost;

    uint256 constant MERGER_PK = 0x1234567890abcdef;
    address mergerAddr;

    uint256 constant AGENT_PK = 0xfedcba0987654321;
    address agentAddr;

    address constant FAKE_TOKEN = address(0x50B);

    function setUp() public {
        mergerAddr = vm.addr(MERGER_PK);
        agentAddr = vm.addr(AGENT_PK);

        mockGda = new MockGDA();
        mockHost = new MockHost();

        pool = new GraffitiPoolHarness(
            FAKE_TOKEN,
            mergerAddr,
            address(mockGda),
            address(mockHost),
            address(0x6DA1) // fake GDA agreement address
        );
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    function _sign(
        address contributor,
        bytes32 commitHash,
        uint128 units,
        uint256 nonce
    ) internal pure returns (bytes memory) {
        bytes32 digest = keccak256(abi.encodePacked(
            contributor, commitHash, units, nonce
        ));
        bytes32 ethHash = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32", digest
        ));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(MERGER_PK, ethHash);
        return abi.encodePacked(r, s, v);
    }

    function _signWithKey(
        uint256 pk,
        address contributor,
        bytes32 commitHash,
        uint128 units,
        uint256 nonce
    ) internal pure returns (bytes memory) {
        bytes32 digest = keccak256(abi.encodePacked(
            contributor, commitHash, units, nonce
        ));
        bytes32 ethHash = keccak256(abi.encodePacked(
            "\x19Ethereum Signed Message:\n32", digest
        ));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, ethHash);
        return abi.encodePacked(r, s, v);
    }

    // ── Happy path ──────────────────────────────────────────────────────

    function test_claim_success() public {
        bytes32 commitHash = bytes32(uint256(0xabc));
        uint128 units = 5;
        uint256 nonce = 1;

        bytes memory sig = _sign(agentAddr, commitHash, units, nonce);

        vm.prank(agentAddr);
        pool.claim(commitHash, units, nonce, sig);

        // State updated
        assertEq(pool.totalMembers(), 1);
        assertTrue(pool.claimed(keccak256(abi.encodePacked(agentAddr, commitHash, units, nonce))));

        // GDA called with correct units
        assertEq(mockGda.lastMember(), agentAddr);
        assertEq(mockGda.lastUnits(), units);

        // Host called (tryConnectPoolFor)
        assertEq(mockHost.callCount(), 1);
    }

    function test_claim_emits_event() public {
        bytes32 commitHash = bytes32(uint256(0xdef));
        uint128 units = 3;
        uint256 nonce = 1;

        bytes memory sig = _sign(agentAddr, commitHash, units, nonce);

        vm.expectEmit(true, false, false, true);
        emit Claimed(agentAddr, commitHash, units);

        vm.prank(agentAddr);
        pool.claim(commitHash, units, nonce, sig);
    }

    function test_claim_different_units() public {
        // 1 unit (minimum)
        bytes memory sig1 = _sign(agentAddr, bytes32(uint256(1)), 1, 1);
        vm.prank(agentAddr);
        pool.claim(bytes32(uint256(1)), 1, 1, sig1);
        assertEq(mockGda.lastUnits(), 1);

        // 10 units (maximum)
        bytes memory sig10 = _sign(agentAddr, bytes32(uint256(2)), 10, 2);
        vm.prank(agentAddr);
        pool.claim(bytes32(uint256(2)), 10, 2, sig10);
        assertEq(mockGda.lastUnits(), 10);
        assertEq(pool.totalMembers(), 2);
    }

    function test_multiple_agents_claim() public {
        address agent2 = vm.addr(0xaaaa);

        bytes memory sig1 = _sign(agentAddr, bytes32(uint256(1)), 5, 1);
        bytes memory sig2 = _sign(agent2, bytes32(uint256(2)), 3, 1);

        vm.prank(agentAddr);
        pool.claim(bytes32(uint256(1)), 5, 1, sig1);

        vm.prank(agent2);
        pool.claim(bytes32(uint256(2)), 3, 1, sig2);

        assertEq(pool.totalMembers(), 2);
    }

    // ── Revert cases ────────────────────────────────────────────────────

    function test_revert_zero_units() public {
        bytes memory sig = _sign(agentAddr, bytes32(uint256(1)), 0, 1);

        vm.prank(agentAddr);
        vm.expectRevert("units 1-10");
        pool.claim(bytes32(uint256(1)), 0, 1, sig);
    }

    function test_revert_units_over_10() public {
        bytes memory sig = _sign(agentAddr, bytes32(uint256(1)), 11, 1);

        vm.prank(agentAddr);
        vm.expectRevert("units 1-10");
        pool.claim(bytes32(uint256(1)), 11, 1, sig);
    }

    function test_revert_double_claim() public {
        bytes32 commitHash = bytes32(uint256(0xabc));
        bytes memory sig = _sign(agentAddr, commitHash, 5, 1);

        vm.prank(agentAddr);
        pool.claim(commitHash, 5, 1, sig);

        vm.prank(agentAddr);
        vm.expectRevert("already claimed");
        pool.claim(commitHash, 5, 1, sig);
    }

    function test_revert_wrong_signer() public {
        // Sign with a random key, not the merger
        bytes memory badSig = _signWithKey(0xBAD, agentAddr, bytes32(uint256(1)), 5, 1);

        vm.prank(agentAddr);
        vm.expectRevert("bad sig");
        pool.claim(bytes32(uint256(1)), 5, 1, badSig);
    }

    function test_revert_wrong_claimer() public {
        // Signature is for agentAddr but rando tries to claim
        bytes memory sig = _sign(agentAddr, bytes32(uint256(1)), 5, 1);

        address rando = vm.addr(0xBADD);
        vm.prank(rando);
        vm.expectRevert("bad sig");
        pool.claim(bytes32(uint256(1)), 5, 1, sig);
    }

    function test_revert_tampered_units() public {
        // Signed for 5 units but trying to claim 10
        bytes memory sig = _sign(agentAddr, bytes32(uint256(1)), 5, 1);

        vm.prank(agentAddr);
        vm.expectRevert("bad sig");
        pool.claim(bytes32(uint256(1)), 10, 1, sig);
    }

    function test_revert_tampered_commit_hash() public {
        bytes memory sig = _sign(agentAddr, bytes32(uint256(1)), 5, 1);

        vm.prank(agentAddr);
        vm.expectRevert("bad sig");
        pool.claim(bytes32(uint256(999)), 5, 1, sig);
    }

    function test_revert_bad_sig_length() public {
        vm.prank(agentAddr);
        vm.expectRevert("bad sig len");
        pool.claim(bytes32(uint256(1)), 5, 1, hex"deadbeef");
    }

    // ── Same agent, different commits (different nonces) ────────────────

    function test_same_agent_multiple_commits() public {
        bytes memory sig1 = _sign(agentAddr, bytes32(uint256(1)), 5, 1);
        bytes memory sig2 = _sign(agentAddr, bytes32(uint256(2)), 3, 2);

        vm.prank(agentAddr);
        pool.claim(bytes32(uint256(1)), 5, 1, sig1);

        vm.prank(agentAddr);
        pool.claim(bytes32(uint256(2)), 3, 2, sig2);

        assertEq(pool.totalMembers(), 2);
        // Last updateMemberUnits call had 3 units
        assertEq(mockGda.lastUnits(), 3);
        assertEq(mockGda.updateCount(), 2);
    }

    // ── Constructor ─────────────────────────────────────────────────────

    function test_constructor_state() public {
        assertEq(pool.trustedMerger(), mergerAddr);
        assertEq(pool.token(), FAKE_TOKEN);
        assertEq(pool.pool(), address(0xBEEF)); // from mock
        assertEq(pool.totalMembers(), 0);
    }
}
