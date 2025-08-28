// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "./ProofPayUsers.sol";

contract ProofPayments is ReentrancyGuard, Ownable {
    using SafeERC20 for IERC20;

    enum PaymentStatus { Pending, Completed, Disputed, Cancelled }
    enum ProofType { None, Text, Photo, zkTLS, Hybrid }

    struct Payment {
        bytes32 id;
        address sender;
        address recipient;
        uint256 amount;
        address token;
        PaymentStatus status;
        ProofType proofType;
        bytes proofData;
        string description;
        uint256 createdAt;
        uint256 completedAt;
        bool requiresProof;
    }

    struct PaymentParams {
        address recipient;
        uint256 amount;
        address token;
        ProofType proofType;
        string description;
        bool requiresProof;
    }

    ProofPayUsers public immutable userContract;
    
    mapping(bytes32 => Payment) public payments;
    mapping(address => bytes32[]) public userPayments;
    mapping(address => uint256) public pendingBalances;
    
    uint256 public constant MAX_DESCRIPTION_LENGTH = 500;
    uint256 public constant MAX_PROOF_SIZE = 10000; // bytes
    uint256 public paymentCounter;
    
    event PaymentCreated(
        bytes32 indexed paymentId,
        address indexed sender,
        address indexed recipient,
        uint256 amount,
        address token,
        ProofType proofType,
        string description
    );
    
    event ProofSubmitted(
        bytes32 indexed paymentId,
        ProofType proofType,
        uint256 timestamp
    );
    
    event PaymentCompleted(
        bytes32 indexed paymentId,
        uint256 timestamp
    );
    
    event PaymentDisputed(
        bytes32 indexed paymentId,
        string reason,
        uint256 timestamp
    );
    
    event PaymentCancelled(
        bytes32 indexed paymentId,
        uint256 timestamp
    );

    constructor(address _userContract) {
        userContract = ProofPayUsers(_userContract);
    }

    modifier validPayment(bytes32 paymentId) {
        require(payments[paymentId].sender != address(0), "Payment does not exist");
        _;
    }

    modifier onlyPaymentParticipant(bytes32 paymentId) {
        Payment memory payment = payments[paymentId];
        require(
            msg.sender == payment.sender || 
            msg.sender == payment.recipient ||
            userContract.isAuthorized(payment.sender, msg.sender) ||
            userContract.isAuthorized(payment.recipient, msg.sender),
            "Not authorized for this payment"
        );
        _;
    }

    function createPayment(PaymentParams calldata params) external nonReentrant returns (bytes32) {
        require(params.recipient != address(0), "Invalid recipient");
        require(params.recipient != msg.sender, "Cannot pay yourself");
        require(params.amount > 0, "Amount must be greater than 0");
        require(bytes(params.description).length <= MAX_DESCRIPTION_LENGTH, "Description too long");

        paymentCounter++;
        bytes32 paymentId = keccak256(abi.encodePacked(
            msg.sender,
            params.recipient,
            params.amount,
            params.token,
            paymentCounter,
            block.timestamp
        ));

        if (params.token == address(0)) {
            require(msg.value == params.amount, "Incorrect ETH amount");
        } else {
            require(msg.value == 0, "Do not send ETH for token payments");
            IERC20(params.token).safeTransferFrom(msg.sender, address(this), params.amount);
        }

        payments[paymentId] = Payment({
            id: paymentId,
            sender: msg.sender,
            recipient: params.recipient,
            amount: params.amount,
            token: params.token,
            status: PaymentStatus.Pending,
            proofType: params.proofType,
            proofData: "",
            description: params.description,
            createdAt: block.timestamp,
            completedAt: 0,
            requiresProof: params.requiresProof
        });

        userPayments[msg.sender].push(paymentId);
        userPayments[params.recipient].push(paymentId);
        pendingBalances[params.recipient] += params.amount;

        emit PaymentCreated(
            paymentId,
            msg.sender,
            params.recipient,
            params.amount,
            params.token,
            params.proofType,
            params.description
        );

        return paymentId;
    }

    function submitProof(bytes32 paymentId, bytes calldata proof) 
        external 
        validPayment(paymentId)
        onlyPaymentParticipant(paymentId)
        nonReentrant 
    {
        Payment storage payment = payments[paymentId];
        require(payment.status == PaymentStatus.Pending, "Payment not pending");
        require(payment.requiresProof, "Payment does not require proof");
        require(proof.length <= MAX_PROOF_SIZE, "Proof too large");
        require(msg.sender == payment.recipient, "Only recipient can submit proof");

        payment.proofData = proof;

        emit ProofSubmitted(paymentId, payment.proofType, block.timestamp);
    }

    function completePayment(bytes32 paymentId) 
        external 
        validPayment(paymentId)
        onlyPaymentParticipant(paymentId)
        nonReentrant 
    {
        Payment storage payment = payments[paymentId];
        require(payment.status == PaymentStatus.Pending, "Payment not pending");
        
        if (payment.requiresProof) {
            require(payment.proofData.length > 0, "Proof required but not submitted");
            require(msg.sender == payment.sender, "Only sender can complete after proof");
        }

        payment.status = PaymentStatus.Completed;
        payment.completedAt = block.timestamp;
        pendingBalances[payment.recipient] -= payment.amount;

        if (payment.token == address(0)) {
            payable(payment.recipient).transfer(payment.amount);
        } else {
            IERC20(payment.token).safeTransfer(payment.recipient, payment.amount);
        }

        emit PaymentCompleted(paymentId, block.timestamp);
    }

    function disputePayment(bytes32 paymentId, string calldata reason) 
        external 
        validPayment(paymentId)
        onlyPaymentParticipant(paymentId)
        nonReentrant 
    {
        Payment storage payment = payments[paymentId];
        require(payment.status == PaymentStatus.Pending, "Payment not pending");
        require(bytes(reason).length > 0 && bytes(reason).length <= 200, "Invalid dispute reason");

        payment.status = PaymentStatus.Disputed;

        emit PaymentDisputed(paymentId, reason, block.timestamp);
    }

    function cancelPayment(bytes32 paymentId) 
        external 
        validPayment(paymentId)
        nonReentrant 
    {
        Payment storage payment = payments[paymentId];
        require(payment.status == PaymentStatus.Pending, "Payment not pending");
        require(msg.sender == payment.sender, "Only sender can cancel");

        payment.status = PaymentStatus.Cancelled;
        pendingBalances[payment.recipient] -= payment.amount;

        if (payment.token == address(0)) {
            payable(payment.sender).transfer(payment.amount);
        } else {
            IERC20(payment.token).safeTransfer(payment.sender, payment.amount);
        }

        emit PaymentCancelled(paymentId, block.timestamp);
    }

    function getUserPayments(address user) external view returns (bytes32[] memory) {
        return userPayments[user];
    }

    function getPayment(bytes32 paymentId) external view returns (Payment memory) {
        return payments[paymentId];
    }
}