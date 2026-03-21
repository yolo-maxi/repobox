// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/RepoBoxNames.sol";
import "../src/RepoBoxMint.sol";

contract RepoBoxNamesTest is Test {
    RepoBoxNames public nftContract;
    RepoBoxMint public mintContract;

    address public owner = address(0x1);
    address public recipient = address(0x2);
    address public user1 = address(0x3);
    address public user2 = address(0x4);

    function setUp() public {
        vm.startPrank(owner);

        // Deploy NFT contract
        nftContract = new RepoBoxNames();

        // Deploy mint contract
        mintContract = new RepoBoxMint(address(nftContract), recipient);

        // Set mint contract
        nftContract.setMintContract(address(mintContract));

        vm.stopPrank();
    }

    // ===== NFT CONTRACT TESTS =====

    function testInitialState() public {
        assertEq(nftContract.name(), "RepoBox Names");
        assertEq(nftContract.symbol(), "RBN");
        assertEq(nftContract.owner(), owner);
        assertEq(nftContract.mintContract(), address(mintContract));
    }

    function testSetMintContract() public {
        address newMintContract = address(0x5);

        vm.startPrank(owner);
        nftContract.setMintContract(newMintContract);
        vm.stopPrank();

        assertEq(nftContract.mintContract(), newMintContract);
    }

    function testSetMintContractOnlyOwner() public {
        address newMintContract = address(0x5);

        vm.startPrank(user1);
        vm.expectRevert();
        nftContract.setMintContract(newMintContract);
        vm.stopPrank();
    }

    function testNameExists() public {
        assertFalse(nftContract.nameExists("test"));

        // Mint a token
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        assertTrue(nftContract.nameExists("test"));
        assertFalse(nftContract.nameExists("other"));
    }

    function testGetTokenId() public {
        uint256 tokenId1 = nftContract.getTokenId("test");
        uint256 tokenId2 = nftContract.getTokenId("test");
        uint256 tokenId3 = nftContract.getTokenId("other");

        // Same name should give same token ID
        assertEq(tokenId1, tokenId2);

        // Different names should give different token IDs
        assertTrue(tokenId1 != tokenId3);
    }

    function testMintThroughMintContract() public {
        vm.deal(user1, 1 ether);

        vm.startPrank(user1);
        uint256 price = mintContract.currentPrice();
        mintContract.mint{value: price}("test", user1);
        vm.stopPrank();

        uint256 tokenId = nftContract.getTokenId("test");

        assertEq(nftContract.ownerOf(tokenId), user1);
        assertEq(nftContract.tokenName(tokenId), "test");
        assertEq(nftContract.resolvedAddress(tokenId), user1);
        assertTrue(nftContract.nameExists("test"));
    }

    function testDirectMintFails() public {
        vm.startPrank(user1);
        vm.expectRevert("Only mint contract can call");
        nftContract.mint(user1, "test", user1);
        vm.stopPrank();
    }

    function testSetResolvedAddress() public {
        // Mint a token
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        uint256 tokenId = nftContract.getTokenId("test");

        // Update resolved address
        vm.startPrank(user1);
        nftContract.setResolvedAddress(tokenId, user2);
        vm.stopPrank();

        assertEq(nftContract.resolvedAddress(tokenId), user2);
    }

    function testSetResolvedAddressOnlyOwner() public {
        // Mint a token
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        uint256 tokenId = nftContract.getTokenId("test");

        // Try to update from different address
        vm.startPrank(user2);
        vm.expectRevert("Only token owner can update");
        nftContract.setResolvedAddress(tokenId, user2);
        vm.stopPrank();
    }

    function testTokenURI() public {
        // Mint a token
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        uint256 tokenId = nftContract.getTokenId("test");
        string memory uri = nftContract.tokenURI(tokenId);

        // Should start with data:application/json;base64,
        assertTrue(bytes(uri).length > 0);
    }

    function testTokenURINonExistent() public {
        vm.expectRevert("Token does not exist");
        nftContract.tokenURI(999);
    }

    function testWithdraw() public {
        // Send some ETH to contract
        vm.deal(address(nftContract), 1 ether);

        uint256 ownerBalanceBefore = owner.balance;

        vm.startPrank(owner);
        nftContract.withdraw();
        vm.stopPrank();

        assertEq(owner.balance, ownerBalanceBefore + 1 ether);
        assertEq(address(nftContract).balance, 0);
    }

    function testWithdrawOnlyOwner() public {
        vm.deal(address(nftContract), 1 ether);

        vm.startPrank(user1);
        vm.expectRevert();
        nftContract.withdraw();
        vm.stopPrank();
    }

    function testWithdrawNoBalance() public {
        vm.startPrank(owner);
        vm.expectRevert("No balance to withdraw");
        nftContract.withdraw();
        vm.stopPrank();
    }

    // ===== MINT CONTRACT TESTS =====

    function testMintContractInitialState() public {
        assertEq(address(mintContract.nftContract()), address(nftContract));
        assertEq(mintContract.recipient(), recipient);
        assertEq(mintContract.totalMinted(), 0);
        assertEq(mintContract.currentPrice(), 0.0005 ether);
    }

    function testBondingCurve() public {
        uint256 basePrice = 0.0005 ether;
        uint256 increment = 0.000005 ether;

        // First mint
        assertEq(mintContract.currentPrice(), basePrice);

        vm.deal(user1, 10 ether);

        // Mint first token
        vm.startPrank(user1);
        mintContract.mint{value: basePrice}("test1", user1);
        vm.stopPrank();

        assertEq(mintContract.totalMinted(), 1);
        assertEq(mintContract.currentPrice(), basePrice + increment);

        // Mint second token
        vm.startPrank(user1);
        mintContract.mint{value: basePrice + increment}("test2", user1);
        vm.stopPrank();

        assertEq(mintContract.totalMinted(), 2);
        assertEq(mintContract.currentPrice(), basePrice + (2 * increment));

        // Test multiple mints
        for (uint256 i = 3; i <= 10; i++) {
            vm.startPrank(user1);
            mintContract.mint{value: mintContract.currentPrice()}(
                string(abi.encodePacked("test", vm.toString(i))),
                user1
            );
            vm.stopPrank();

            assertEq(mintContract.totalMinted(), i);
            assertEq(mintContract.currentPrice(), basePrice + (i * increment));
        }
    }

    function testMintBondingCurveLargeNumbers() public {
        uint256 basePrice = 0.0005 ether;
        uint256 increment = 0.000005 ether;

        // Test pricing after 5 mints
        vm.deal(user1, 10 ether);

        for (uint256 i = 1; i <= 5; i++) {
            uint256 expectedPrice = basePrice + ((i-1) * increment);
            assertEq(mintContract.currentPrice(), expectedPrice);

            vm.startPrank(user1);
            mintContract.mint{value: expectedPrice}(
                string(abi.encodePacked("test", vm.toString(i))),
                user1
            );
            vm.stopPrank();
        }

        // After 5 mints, check the pricing
        assertEq(mintContract.totalMinted(), 5);
        assertEq(mintContract.currentPrice(), basePrice + (5 * increment));

        // Next price should be for the 6th mint
        assertEq(mintContract.nextPrice(), basePrice + (6 * increment));
    }

    function testNameValidation() public {
        // Valid names
        assertTrue(mintContract.validateName("abc"));
        assertTrue(mintContract.validateName("test123"));
        assertTrue(mintContract.validateName("repo-box"));
        assertTrue(mintContract.validateName("a1b2c3"));

        // Invalid names - too short
        assertFalse(mintContract.validateName("ab"));
        assertFalse(mintContract.validateName("a"));
        assertFalse(mintContract.validateName(""));

        // Invalid names - too long
        assertFalse(mintContract.validateName("abcdefghijklmnopqrstuvwxyz1234567"));

        // Invalid names - start/end with hyphen
        assertFalse(mintContract.validateName("-test"));
        assertFalse(mintContract.validateName("test-"));
        assertFalse(mintContract.validateName("-test-"));

        // Invalid names - invalid characters
        assertFalse(mintContract.validateName("test_name"));
        assertFalse(mintContract.validateName("test.name"));
        assertFalse(mintContract.validateName("test@name"));
        assertFalse(mintContract.validateName("test name"));
        assertFalse(mintContract.validateName("Test"));
        assertFalse(mintContract.validateName("TEST"));
    }

    function testMintInsufficientPayment() public {
        vm.deal(user1, 1 ether);

        vm.startPrank(user1);
        vm.expectRevert("Insufficient payment");
        mintContract.mint{value: 0.0001 ether}("test", user1);
        vm.stopPrank();
    }

    function testMintInvalidName() public {
        vm.deal(user1, 1 ether);

        // Test empty name - should revert
        vm.startPrank(user1);
        try mintContract.mint{value: mintContract.currentPrice()}("", user1) {
            fail("Expected revert for empty name");
        } catch Error(string memory reason) {
            assertEq(reason, "Invalid name format");
        }
        vm.stopPrank();

        // Test too short name
        vm.startPrank(user1);
        try mintContract.mint{value: mintContract.currentPrice()}("ab", user1) {
            fail("Expected revert for short name");
        } catch Error(string memory reason) {
            assertEq(reason, "Invalid name format");
        }
        vm.stopPrank();

        // Test name starting with hyphen
        vm.startPrank(user1);
        try mintContract.mint{value: mintContract.currentPrice()}("-test", user1) {
            fail("Expected revert for name starting with hyphen");
        } catch Error(string memory reason) {
            assertEq(reason, "Invalid name format");
        }
        vm.stopPrank();
    }

    function testMintDuplicateName() public {
        vm.deal(user1, 1 ether);

        // First mint
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        // Try to mint same name - should revert
        vm.deal(user2, 1 ether);
        vm.startPrank(user2);
        uint256 currentPriceForSecond = mintContract.currentPrice();
        try mintContract.mint{value: currentPriceForSecond}("test", user2) {
            fail("Expected revert for duplicate name");
        } catch Error(string memory reason) {
            assertEq(reason, "Name already exists");
        }
        vm.stopPrank();
    }

    function testMintRefund() public {
        uint256 price = mintContract.currentPrice();
        uint256 overpayment = 0.1 ether;

        vm.deal(user1, price + overpayment);
        uint256 balanceBefore = user1.balance;

        vm.startPrank(user1);
        mintContract.mint{value: price + overpayment}("test", user1);
        vm.stopPrank();

        uint256 balanceAfter = user1.balance;

        // Should have paid exactly the price
        assertEq(balanceBefore - balanceAfter, price);

        // Recipient should have received the price
        assertEq(recipient.balance, price);
    }

    function testGetPricingInfo() public {
        (
            uint256 basePrice,
            uint256 increment,
            uint256 current,
            uint256 next,
            uint256 minted
        ) = mintContract.getPricingInfo();

        assertEq(basePrice, 0.0005 ether);
        assertEq(increment, 0.000005 ether);
        assertEq(current, 0.0005 ether);
        assertEq(next, 0.000505 ether); // 0.0005 + (1 * 0.000005) = 0.000505
        assertEq(minted, 0);

        // After one mint
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: current}("test", user1);
        vm.stopPrank();

        (basePrice, increment, current, next, minted) = mintContract.getPricingInfo();

        assertEq(basePrice, 0.0005 ether);
        assertEq(increment, 0.000005 ether);
        assertEq(current, 0.000505 ether);
        assertEq(next, 0.00051 ether); // 0.0005 + (2 * 0.000005) = 0.00051
        assertEq(minted, 1);
    }

    function testNextPrice() public {
        assertEq(mintContract.nextPrice(), 0.000505 ether);

        // After one mint
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        mintContract.mint{value: mintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        assertEq(mintContract.nextPrice(), 0.00051 ether);
    }

    function testEmergencyWithdraw() public {
        // Send some ETH to mint contract
        vm.deal(address(mintContract), 1 ether);

        uint256 ownerBalanceBefore = owner.balance;

        vm.startPrank(owner);
        mintContract.emergencyWithdraw();
        vm.stopPrank();

        assertEq(owner.balance, ownerBalanceBefore + 1 ether);
        assertEq(address(mintContract).balance, 0);
    }

    function testEmergencyWithdrawOnlyOwner() public {
        vm.deal(address(mintContract), 1 ether);

        vm.startPrank(user1);
        vm.expectRevert("Only NFT contract owner");
        mintContract.emergencyWithdraw();
        vm.stopPrank();
    }

    function testSwapMintContract() public {
        // Deploy new mint contract
        vm.startPrank(owner);
        RepoBoxMint newMintContract = new RepoBoxMint(address(nftContract), recipient);

        // Update mint contract
        nftContract.setMintContract(address(newMintContract));
        vm.stopPrank();

        // Old mint contract should no longer work
        vm.deal(user1, 1 ether);
        vm.startPrank(user1);
        try mintContract.mint{value: mintContract.currentPrice()}("test", user1) {
            fail("Expected revert for old mint contract");
        } catch Error(string memory reason) {
            assertEq(reason, "Only mint contract can call");
        }
        vm.stopPrank();

        // New mint contract should work
        vm.startPrank(user1);
        newMintContract.mint{value: newMintContract.currentPrice()}("test", user1);
        vm.stopPrank();

        assertTrue(nftContract.nameExists("test"));
    }

    // ===== INTEGRATION TESTS =====

    function testFullWorkflow() public {
        vm.deal(user1, 10 ether);
        vm.deal(user2, 10 ether);

        // User1 mints first token
        vm.startPrank(user1);
        uint256 price1 = mintContract.currentPrice();
        mintContract.mint{value: price1}("alice", user1);
        vm.stopPrank();

        uint256 tokenId1 = nftContract.getTokenId("alice");
        assertEq(nftContract.ownerOf(tokenId1), user1);
        assertEq(nftContract.resolvedAddress(tokenId1), user1);

        // User2 mints second token (higher price due to bonding curve)
        vm.startPrank(user2);
        uint256 price2 = mintContract.currentPrice();
        assertTrue(price2 > price1);
        mintContract.mint{value: price2}("bob", user2);
        vm.stopPrank();

        uint256 tokenId2 = nftContract.getTokenId("bob");
        assertEq(nftContract.ownerOf(tokenId2), user2);
        assertEq(nftContract.resolvedAddress(tokenId2), user2);

        // User1 updates their resolved address
        vm.startPrank(user1);
        nftContract.setResolvedAddress(tokenId1, user2);
        vm.stopPrank();

        assertEq(nftContract.resolvedAddress(tokenId1), user2);

        // Verify recipient received payments
        assertEq(recipient.balance, price1 + price2);

        // Verify total minted
        assertEq(mintContract.totalMinted(), 2);
    }
}