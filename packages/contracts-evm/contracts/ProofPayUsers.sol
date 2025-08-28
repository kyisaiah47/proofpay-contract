// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract ProofPayUsers is Ownable, ReentrancyGuard {
    struct User {
        string username;
        bool isRegistered;
        uint256 createdAt;
    }
    
    mapping(address => User) public users;
    mapping(string => address) public usernameToAddress;
    mapping(address => mapping(address => bool)) public authorizedAddresses;
    
    event UserRegistered(address indexed userAddress, string username, uint256 timestamp);
    event AuthorizedAddressAdded(address indexed user, address indexed authorizedAddress);
    event AuthorizedAddressRemoved(address indexed user, address indexed authorizedAddress);
    
    modifier onlyRegistered() {
        require(users[msg.sender].isRegistered, "User not registered");
        _;
    }
    
    modifier validUsername(string calldata username) {
        require(bytes(username).length > 0 && bytes(username).length <= 32, "Invalid username length");
        require(usernameToAddress[username] == address(0), "Username already taken");
        _;
    }
    
    function registerUser(string calldata username) external validUsername(username) nonReentrant {
        require(!users[msg.sender].isRegistered, "User already registered");
        
        users[msg.sender] = User({
            username: username,
            isRegistered: true,
            createdAt: block.timestamp
        });
        
        usernameToAddress[username] = msg.sender;
        
        emit UserRegistered(msg.sender, username, block.timestamp);
    }
    
    function addAuthorizedAddress(address newAddress) external onlyRegistered nonReentrant {
        require(newAddress != address(0), "Invalid address");
        require(newAddress != msg.sender, "Cannot authorize own address");
        require(!authorizedAddresses[msg.sender][newAddress], "Address already authorized");
        
        authorizedAddresses[msg.sender][newAddress] = true;
        
        emit AuthorizedAddressAdded(msg.sender, newAddress);
    }
    
    function removeAuthorizedAddress(address addressToRemove) external onlyRegistered nonReentrant {
        require(authorizedAddresses[msg.sender][addressToRemove], "Address not authorized");
        
        authorizedAddresses[msg.sender][addressToRemove] = false;
        
        emit AuthorizedAddressRemoved(msg.sender, addressToRemove);
    }
    
    function isAuthorized(address user, address addressToCheck) external view returns (bool) {
        return user == addressToCheck || authorizedAddresses[user][addressToCheck];
    }
    
    function getUserByUsername(string calldata username) external view returns (address) {
        return usernameToAddress[username];
    }
    
    function isUsernameAvailable(string calldata username) external view returns (bool) {
        return usernameToAddress[username] == address(0);
    }
}