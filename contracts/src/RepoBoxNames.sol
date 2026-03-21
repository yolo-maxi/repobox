// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "@openzeppelin/contracts/utils/Base64.sol";

/**
 * @title RepoBoxNames
 * @dev NFT contract representing {name}.repo.box ENS subdomains
 * Each NFT represents ownership of a subdomain and its resolution address
 */
contract RepoBoxNames is ERC721, Ownable {
    using Strings for uint256;

    // The address that can mint new tokens (mint contract)
    address public mintContract;

    // Mapping from token ID to the name string
    mapping(uint256 => string) public tokenName;

    // Mapping from token ID to the resolved EVM address
    mapping(uint256 => address) public resolvedAddress;

    // Mapping from name hash to token ID for uniqueness checks
    mapping(bytes32 => uint256) public nameToToken;

    // Mapping to track if a name exists (to handle tokenId = 0 case)
    mapping(bytes32 => bool) private _nameExists;

    // Events
    event ResolvedAddressUpdated(uint256 indexed tokenId, address indexed newAddress);
    event MintContractUpdated(address indexed newMintContract);

    constructor() ERC721("RepoBox Names", "RBN") Ownable(msg.sender) {}

    /**
     * @dev Only the approved mint contract can call certain functions
     */
    modifier onlyMintContract() {
        require(msg.sender == mintContract, "Only mint contract can call");
        _;
    }

    /**
     * @dev Set the address that can mint new tokens
     * @param _mintContract Address of the mint contract
     */
    function setMintContract(address _mintContract) external onlyOwner {
        mintContract = _mintContract;
        emit MintContractUpdated(_mintContract);
    }

    /**
     * @dev Check if a name already exists
     * @param name The name to check
     * @return bool True if name exists, false otherwise
     */
    function nameExists(string memory name) public view returns (bool) {
        bytes32 nameHash = keccak256(abi.encodePacked(name));
        return _nameExists[nameHash];
    }

    /**
     * @dev Get token ID for a name
     * @param name The name to get token ID for
     * @return uint256 The token ID (deterministic based on name)
     */
    function getTokenId(string memory name) public pure returns (uint256) {
        return uint256(keccak256(abi.encodePacked(name)));
    }

    /**
     * @dev Mint a new NFT representing a subdomain
     * @param to Address to mint the token to
     * @param name The subdomain name (without .repo.box)
     * @param _resolvedAddress The EVM address this name resolves to
     */
    function mint(
        address to,
        string memory name,
        address _resolvedAddress
    ) external onlyMintContract {
        require(bytes(name).length > 0, "Name cannot be empty");

        bytes32 nameHash = keccak256(abi.encodePacked(name));
        require(!_nameExists[nameHash], "Name already exists");

        uint256 tokenId = getTokenId(name);

        // Store the mappings
        tokenName[tokenId] = name;
        resolvedAddress[tokenId] = _resolvedAddress;
        nameToToken[nameHash] = tokenId;
        _nameExists[nameHash] = true;

        // Mint the NFT
        _mint(to, tokenId);
    }

    /**
     * @dev Update the resolved address for a token (only token owner can call)
     * @param tokenId The token ID
     * @param _resolvedAddress New address to resolve to
     */
    function setResolvedAddress(uint256 tokenId, address _resolvedAddress) external {
        require(ownerOf(tokenId) == msg.sender, "Only token owner can update");

        resolvedAddress[tokenId] = _resolvedAddress;
        emit ResolvedAddressUpdated(tokenId, _resolvedAddress);
    }

    /**
     * @dev Generate on-chain metadata for a token
     * @param tokenId The token ID
     * @return string Base64 encoded JSON metadata
     */
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(_exists(tokenId), "Token does not exist");

        string memory name = tokenName[tokenId];
        address resolved = resolvedAddress[tokenId];

        string memory json = Base64.encode(
            bytes(
                string(
                    abi.encodePacked(
                        '{"name": "',
                        name,
                        '.repo.box", "description": "ENS subdomain NFT for repo.box", "attributes": [{"trait_type": "Name", "value": "',
                        name,
                        '"}, {"trait_type": "Resolved Address", "value": "',
                        Strings.toHexString(uint160(resolved), 20),
                        '"}, {"trait_type": "Full Domain", "value": "',
                        name,
                        '.repo.box"}]}'
                    )
                )
            )
        );

        return string(abi.encodePacked("data:application/json;base64,", json));
    }

    /**
     * @dev Withdraw any stuck ETH from the contract (owner only)
     */
    function withdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No balance to withdraw");

        (bool success, ) = payable(owner()).call{value: balance}("");
        require(success, "Withdrawal failed");
    }

    /**
     * @dev Check if a token exists
     * @param tokenId The token ID to check
     * @return bool True if token exists
     */
    function _exists(uint256 tokenId) internal view returns (bool) {
        return _ownerOf(tokenId) != address(0);
    }
}