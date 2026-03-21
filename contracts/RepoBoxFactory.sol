// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./RepoBoxNames.sol";
import "./RepoBoxMint.sol";

/// @title RepoBoxFactory — One-click deploy for the whole ENS subdomain system
/// @notice Deploys RepoBoxNames + RepoBoxMint and wires them together in a single tx
contract RepoBoxFactory {
    event Deployed(address names, address mint);

    /// @notice Deploy everything
    /// @param owner Admin of the NFT contract (0xFran.eth)
    /// @param recipient Revenue recipient for mints (Ocean's wallet)
    /// @return names The NFT contract address
    /// @return mint The mint contract address
    function deploy(address owner, address payable recipient)
        external
        returns (address names, address mint)
    {
        // 1. Deploy NFT contract (owned by this factory temporarily)
        RepoBoxNames n = new RepoBoxNames(address(this));

        // 2. Deploy mint contract pointing at NFT
        RepoBoxMint m = new RepoBoxMint(address(n), recipient);

        // 3. Wire mint contract into NFT
        n.setMintContract(address(m));

        // 4. Transfer ownership to the real owner
        n.transferOwnership(owner);

        emit Deployed(address(n), address(m));
        return (address(n), address(m));
    }
}
