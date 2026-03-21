// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "./RepoBoxNames.sol";

/**
 * @title RepoBoxMint
 * @dev Mint contract for RepoBoxNames with bonding curve pricing
 * Implements a linear bonding curve where price increases with each mint
 */
contract RepoBoxMint is ReentrancyGuard {
    // The NFT contract this minter interacts with
    RepoBoxNames public immutable nftContract;

    // The recipient address that receives payments
    address public immutable recipient;

    // Base price for the first mint (0.0005 ETH)
    uint256 public constant BASE_PRICE = 0.0005 ether;

    // Price increment per mint (0.000005 ETH)
    uint256 public constant PRICE_INCREMENT = 0.000005 ether;

    // Total number of tokens minted through this contract
    uint256 public totalMinted;

    // Events
    event NameMinted(address indexed to, string name, uint256 price, uint256 tokenId);

    /**
     * @dev Constructor
     * @param _nftContract Address of the RepoBoxNames NFT contract
     * @param _recipient Address that receives the ETH payments
     */
    constructor(address _nftContract, address _recipient) {
        require(_nftContract != address(0), "Invalid NFT contract");
        require(_recipient != address(0), "Invalid recipient");

        nftContract = RepoBoxNames(_nftContract);
        recipient = _recipient;
    }

    /**
     * @dev Calculate the current price for minting
     * @return uint256 The current price in wei
     */
    function currentPrice() public view returns (uint256) {
        return BASE_PRICE + (totalMinted * PRICE_INCREMENT);
    }

    /**
     * @dev Validate a subdomain name
     * @param name The name to validate
     * @return bool True if valid, false otherwise
     */
    function validateName(string memory name) public pure returns (bool) {
        bytes memory nameBytes = bytes(name);
        uint256 length = nameBytes.length;

        // Check length constraints
        if (length < 3 || length > 32) {
            return false;
        }

        // Check first and last character are not hyphens
        if (nameBytes[0] == 0x2D || nameBytes[length - 1] == 0x2D) {
            return false;
        }

        // Check all characters are valid (a-z, 0-9, hyphen)
        for (uint256 i = 0; i < length; i++) {
            bytes1 char = nameBytes[i];

            // Allow lowercase letters (a-z)
            if (char >= 0x61 && char <= 0x7A) {
                continue;
            }

            // Allow digits (0-9)
            if (char >= 0x30 && char <= 0x39) {
                continue;
            }

            // Allow hyphen (but not at start/end, already checked)
            if (char == 0x2D) {
                continue;
            }

            // Invalid character
            return false;
        }

        return true;
    }

    /**
     * @dev Mint a new subdomain NFT
     * @param name The subdomain name (without .repo.box)
     * @param resolvedAddress The EVM address this name should resolve to
     */
    function mint(string memory name, address resolvedAddress)
        external
        payable
        nonReentrant
    {
        // Validate the name
        require(validateName(name), "Invalid name format");

        // Check if name already exists
        require(!nftContract.nameExists(name), "Name already exists");

        // Check payment
        uint256 price = currentPrice();
        require(msg.value >= price, "Insufficient payment");

        // Calculate refund if overpaid
        uint256 refund = msg.value - price;

        // Increment total minted before external calls
        totalMinted++;

        // Mint the NFT
        uint256 tokenId = nftContract.getTokenId(name);
        nftContract.mint(msg.sender, name, resolvedAddress);

        // Send payment to recipient
        (bool success, ) = payable(recipient).call{value: price}("");
        require(success, "Payment transfer failed");

        // Refund excess payment
        if (refund > 0) {
            (bool refundSuccess, ) = payable(msg.sender).call{value: refund}("");
            require(refundSuccess, "Refund failed");
        }

        emit NameMinted(msg.sender, name, price, tokenId);
    }

    /**
     * @dev Get the price for the next mint
     * @return uint256 The price for the next token
     */
    function nextPrice() external view returns (uint256) {
        return BASE_PRICE + ((totalMinted + 1) * PRICE_INCREMENT);
    }

    /**
     * @dev Get pricing information
     * @return basePrice The base price
     * @return increment The price increment per mint
     * @return current The current price
     * @return next The next price
     * @return minted Total minted so far
     */
    function getPricingInfo()
        external
        view
        returns (
            uint256 basePrice,
            uint256 increment,
            uint256 current,
            uint256 next,
            uint256 minted
        )
    {
        basePrice = BASE_PRICE;
        increment = PRICE_INCREMENT;
        current = currentPrice();
        next = BASE_PRICE + ((totalMinted + 1) * PRICE_INCREMENT);
        minted = totalMinted;
    }

    /**
     * @dev Emergency function to recover stuck ETH (should not normally happen)
     * Only callable by the NFT contract owner
     */
    function emergencyWithdraw() external {
        require(msg.sender == nftContract.owner(), "Only NFT contract owner");

        uint256 balance = address(this).balance;
        if (balance > 0) {
            (bool success, ) = payable(msg.sender).call{value: balance}("");
            require(success, "Emergency withdrawal failed");
        }
    }
}