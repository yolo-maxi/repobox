// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/// @title RepoBoxMint — Bonding curve minter for repo.box ENS subdomains
/// @notice Swappable mint contract. Owner of RepoBoxNames can replace with new rules.
/// @dev Price starts at 0.0005 ETH, increases by 0.000005 ETH per mint.
contract RepoBoxMint {

    /// @dev The NFT contract we mint through
    IRepoBoxNames public immutable names;

    /// @dev ETH revenue recipient
    address payable public immutable recipient;

    /// @dev Pricing
    uint256 public constant BASE_PRICE = 0.0005 ether;
    uint256 public constant PRICE_INCREMENT = 0.000005 ether;

    /// @dev Mint counter
    uint256 public totalMinted;

    event NameMinted(address indexed buyer, string name, uint256 price, uint256 mintNumber);

    error InsufficientPayment();
    error InvalidName();
    error NameTaken();
    error TransferFailed();

    constructor(address _names, address payable _recipient) {
        names = IRepoBoxNames(_names);
        recipient = _recipient;
    }

    /// @notice Current mint price
    function currentPrice() public view returns (uint256) {
        return BASE_PRICE + (totalMinted * PRICE_INCREMENT);
    }

    /// @notice Mint a {name}.repo.box subdomain NFT
    /// @param name The desired subdomain (e.g. "alice" for alice.repo.box)
    /// @param resolvedAddress The EVM address this name should resolve to
    function mint(string calldata name, address resolvedAddress) external payable {
        // Validate name
        _validateName(name);

        // Check not taken
        if (names.nameExists(name)) revert NameTaken();

        // Check payment
        uint256 price = currentPrice();
        if (msg.value < price) revert InsufficientPayment();

        // Mint
        totalMinted++;
        names.mint(msg.sender, name, resolvedAddress);

        // Forward payment to recipient
        (bool ok,) = recipient.call{value: price}("");
        if (!ok) revert TransferFailed();

        // Refund excess
        if (msg.value > price) {
            (bool refundOk,) = msg.sender.call{value: msg.value - price}("");
            if (!refundOk) revert TransferFailed();
        }

        emit NameMinted(msg.sender, name, price, totalMinted);
    }

    /// @dev Validate name: lowercase a-z, 0-9, hyphens. Min 3, max 32. No leading/trailing hyphens.
    function _validateName(string calldata name) private pure {
        bytes memory b = bytes(name);
        if (b.length < 3 || b.length > 32) revert InvalidName();
        if (b[0] == 0x2d || b[b.length - 1] == 0x2d) revert InvalidName(); // no leading/trailing hyphen

        for (uint256 i = 0; i < b.length; i++) {
            bytes1 c = b[i];
            bool valid = (c >= 0x61 && c <= 0x7a) || // a-z
                         (c >= 0x30 && c <= 0x39) || // 0-9
                         c == 0x2d;                   // hyphen
            if (!valid) revert InvalidName();
        }
    }
}

interface IRepoBoxNames {
    function mint(address to, string calldata name, address resolvedAddress) external returns (uint256);
    function nameExists(string calldata name) external view returns (bool);
}
