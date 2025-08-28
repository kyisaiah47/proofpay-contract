// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./ProofPayUsers.sol";
import "./ProofPayments.sol";

contract ProofPay is ReentrancyGuard, Ownable {
    ProofPayUsers public immutable userContract;
    ProofPayments public immutable paymentContract;
    
    string public constant VERSION = "1.0.0";
    uint256 public totalUsers;
    uint256 public totalPayments;
    uint256 public totalVolume;
    
    event ContractInitialized(address userContract, address paymentContract);
    event StatsUpdated(uint256 totalUsers, uint256 totalPayments, uint256 totalVolume);
    
    constructor() {
        userContract = new ProofPayUsers();
        paymentContract = new ProofPayments(address(userContract));
        
        emit ContractInitialized(address(userContract), address(paymentContract));
    }
    
    function registerUser(string calldata username) external {
        userContract.registerUser(username);
        totalUsers++;
        emit StatsUpdated(totalUsers, totalPayments, totalVolume);
    }
    
    function createPayment(ProofPayments.PaymentParams calldata params) external payable returns (bytes32) {
        bytes32 paymentId = paymentContract.createPayment{value: msg.value}(params);
        totalPayments++;
        totalVolume += params.amount;
        emit StatsUpdated(totalUsers, totalPayments, totalVolume);
        return paymentId;
    }
    
    function getContractAddresses() external view returns (address, address) {
        return (address(userContract), address(paymentContract));
    }
    
    function getStats() external view returns (uint256, uint256, uint256) {
        return (totalUsers, totalPayments, totalVolume);
    }
}