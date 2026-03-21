// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./lib/openzeppelin-contracts/contracts/token/ERC721/ERC721.sol";
import "./lib/openzeppelin-contracts/contracts/access/Ownable.sol";
import "./lib/openzeppelin-contracts/contracts/utils/Strings.sol";
import "./lib/openzeppelin-contracts/contracts/utils/Base64.sol";

/// @title RepoBoxNames — ENS subdomain NFTs for repo.box
/// @notice Each NFT represents a {name}.repo.box ENS subname
/// @dev Owner (Fran) controls admin. Only approved mint contract can mint.
contract RepoBoxNames is ERC721, Ownable {
    using Strings for uint256;
    using Strings for address;

    /// @dev Approved mint contract (swappable by owner)
    address public mintContract;

    /// @dev Token data
    mapping(uint256 => string) public tokenName;
    mapping(uint256 => address) public resolvedAddress;
    mapping(bytes32 => uint256) public nameToToken;
    uint256 public totalSupply;

    event MintContractUpdated(address indexed newMintContract);
    event ResolvedAddressUpdated(uint256 indexed tokenId, address indexed newAddress);

    error OnlyMintContract();
    error NameTaken();
    error NotTokenOwner();

    constructor(address _owner) ERC721("repo.box Names", "REPOBOX") Ownable(_owner) {}

    /// @notice Set the approved mint contract (owner only)
    function setMintContract(address _mintContract) external onlyOwner {
        mintContract = _mintContract;
        emit MintContractUpdated(_mintContract);
    }

    /// @notice Mint a new name NFT (only callable by mint contract)
    function mint(address to, string calldata name, address _resolvedAddress) external returns (uint256) {
        if (msg.sender != mintContract) revert OnlyMintContract();

        bytes32 nameHash = keccak256(bytes(name));
        if (nameToToken[nameHash] != 0) revert NameTaken();

        // tokenId = 1-indexed to distinguish from empty mapping
        totalSupply++;
        uint256 tokenId = totalSupply;

        _safeMint(to, tokenId);
        tokenName[tokenId] = name;
        resolvedAddress[tokenId] = _resolvedAddress;
        nameToToken[nameHash] = tokenId;

        return tokenId;
    }

    /// @notice Check if a name is already taken
    function nameExists(string calldata name) external view returns (bool) {
        return nameToToken[keccak256(bytes(name))] != 0;
    }

    /// @notice Token owner can update where their name resolves to
    function setResolvedAddress(uint256 tokenId, address addr) external {
        if (ownerOf(tokenId) != msg.sender) revert NotTokenOwner();
        resolvedAddress[tokenId] = addr;
        emit ResolvedAddressUpdated(tokenId, addr);
    }

    /// @notice On-chain metadata
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        ownerOf(tokenId); // reverts if doesn't exist

        string memory name = tokenName[tokenId];
        address addr = resolvedAddress[tokenId];

        string memory json = string(abi.encodePacked(
            '{"name":"', name, '.repo.box",',
            '"description":"ENS subdomain for repo.box git hosting",',
            '"attributes":[',
                '{"trait_type":"name","value":"', name, '"},',
                '{"trait_type":"resolves_to","value":"', Strings.toHexString(addr), '"}',
            ']}'
        ));

        return string(abi.encodePacked(
            "data:application/json;base64,",
            Base64.encode(bytes(json))
        ));
    }

    /// @notice Withdraw any stuck ETH (owner only, safety valve)
    function withdraw() external onlyOwner {
        (bool ok,) = owner().call{value: address(this).balance}("");
        require(ok);
    }
}
