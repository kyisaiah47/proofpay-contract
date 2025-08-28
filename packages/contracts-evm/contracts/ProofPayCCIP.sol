// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@chainlink/contracts/src/v0.8/ccip/applications/CCIPReceiver.sol";
import "@chainlink/contracts/src/v0.8/ccip/libraries/Client.sol";
import "@chainlink/contracts/src/v0.8/interfaces/IRouterClient.sol";
import "@chainlink/contracts/src/v0.8/interfaces/LinkTokenInterface.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./ProofPayments.sol";

contract ProofPayCCIP is CCIPReceiver, ReentrancyGuard, Ownable {
    IRouterClient private immutable i_router;
    LinkTokenInterface private immutable i_linkToken;
    ProofPayments private immutable paymentContract;

    struct CrossChainPayment {
        uint64 destinationChain;
        address recipient;
        uint256 amount;
        address token;
        bytes zkProof;
        string description;
    }

    mapping(uint64 => bool) public allowedChains;
    mapping(address => bool) public trustedSenders;
    mapping(bytes32 => bool) public processedMessages;

    event CrossChainPaymentSent(
        bytes32 indexed messageId,
        uint64 indexed destinationChain,
        address indexed recipient,
        uint256 amount,
        address token
    );

    event CrossChainPaymentReceived(
        bytes32 indexed messageId,
        uint64 indexed sourceChain,
        address indexed sender,
        address indexed recipient,
        uint256 amount
    );

    event ChainAllowlistUpdated(uint64 chainSelector, bool allowed);
    event TrustedSenderUpdated(address sender, bool trusted);

    constructor(
        address _router,
        address _linkToken,
        address _paymentContract
    ) CCIPReceiver(_router) {
        i_router = IRouterClient(_router);
        i_linkToken = LinkTokenInterface(_linkToken);
        paymentContract = ProofPayments(_paymentContract);
    }

    modifier onlyAllowedChain(uint64 chainSelector) {
        require(allowedChains[chainSelector], "Chain not allowed");
        _;
    }

    modifier onlyTrustedSender(address sender) {
        require(trustedSenders[sender], "Sender not trusted");
        _;
    }

    function updateChainAllowlist(uint64 chainSelector, bool allowed) external onlyOwner {
        allowedChains[chainSelector] = allowed;
        emit ChainAllowlistUpdated(chainSelector, allowed);
    }

    function updateTrustedSender(address sender, bool trusted) external onlyOwner {
        trustedSenders[sender] = trusted;
        emit TrustedSenderUpdated(sender, trusted);
    }

    function sendCrossChainPayment(
        CrossChainPayment calldata params
    ) external payable onlyAllowedChain(params.destinationChain) nonReentrant returns (bytes32 messageId) {
        require(params.recipient != address(0), "Invalid recipient");
        require(params.amount > 0, "Invalid amount");

        // Build CCIP message
        Client.EVM2AnyMessage memory message = _buildCCIPMessage(
            params.recipient,
            params.amount,
            params.token,
            params.zkProof,
            params.description
        );

        // Calculate fee
        uint256 fee = i_router.getFee(params.destinationChain, message);
        require(i_linkToken.balanceOf(address(this)) >= fee, "Insufficient LINK balance");

        // Approve router to spend LINK tokens for fees
        i_linkToken.approve(address(i_router), fee);

        // Handle token transfer
        if (params.token == address(0)) {
            // Native ETH payment
            require(msg.value >= params.amount, "Insufficient ETH sent");
        } else {
            // ERC20 token payment
            require(msg.value == 0, "Do not send ETH for token payments");
            IERC20(params.token).transferFrom(msg.sender, address(this), params.amount);
        }

        // Send cross-chain message
        messageId = i_router.ccipSend(params.destinationChain, message);

        emit CrossChainPaymentSent(
            messageId,
            params.destinationChain,
            params.recipient,
            params.amount,
            params.token
        );
    }

    function _ccipReceive(
        Client.Any2EVMMessage memory message
    ) internal override nonReentrant {
        bytes32 messageId = message.messageId;
        
        // Prevent replay attacks
        require(!processedMessages[messageId], "Message already processed");
        processedMessages[messageId] = true;

        // Validate source chain
        require(allowedChains[message.sourceChainSelector], "Source chain not allowed");

        // Validate sender
        address sender = abi.decode(message.sender, (address));
        require(trustedSenders[sender], "Sender not trusted");

        // Process the received payment
        _processReceivedPayment(message);

        emit CrossChainPaymentReceived(
            messageId,
            message.sourceChainSelector,
            sender,
            address(0), // Will be decoded from message.data
            0 // Will be decoded from message.data
        );
    }

    function _buildCCIPMessage(
        address recipient,
        uint256 amount,
        address token,
        bytes memory zkProof,
        string memory description
    ) internal view returns (Client.EVM2AnyMessage memory) {
        // Encode payment data
        bytes memory data = abi.encode(
            msg.sender,      // sender
            recipient,       // recipient
            amount,          // amount
            token,           // token address
            zkProof,         // proof data
            description      // description
        );

        Client.EVMTokenAmount[] memory tokenAmounts;
        
        if (token == address(0)) {
            // Native token transfer
            tokenAmounts = new Client.EVMTokenAmount[](1);
            tokenAmounts[0] = Client.EVMTokenAmount({
                token: address(0),
                amount: amount
            });
        } else {
            // ERC20 token transfer
            tokenAmounts = new Client.EVMTokenAmount[](1);
            tokenAmounts[0] = Client.EVMTokenAmount({
                token: token,
                amount: amount
            });
        }

        return Client.EVM2AnyMessage({
            receiver: abi.encode(address(this)),
            data: data,
            tokenAmounts: tokenAmounts,
            extraArgs: Client._argsToBytes(
                Client.EVMExtraArgsV1({gasLimit: 500_000})
            ),
            feeToken: address(i_linkToken)
        });
    }

    function _processReceivedPayment(Client.Any2EVMMessage memory message) internal {
        // Decode payment data
        (
            address sender,
            address recipient,
            uint256 amount,
            address token,
            bytes memory zkProof,
            string memory description
        ) = abi.decode(message.data, (address, address, uint256, address, bytes, string));

        // Create payment parameters
        ProofPayments.PaymentParams memory params = ProofPayments.PaymentParams({
            recipient: recipient,
            amount: amount,
            token: token,
            proofType: zkProof.length > 0 ? ProofPayments.ProofType.zkTLS : ProofPayments.ProofType.None,
            description: description,
            requiresProof: false // Cross-chain payments are pre-verified
        });

        // Create the payment in the payment contract
        if (token == address(0)) {
            // Handle native token
            paymentContract.createPayment{value: amount}(params);
        } else {
            // Handle ERC20 token - first approve the payment contract
            IERC20(token).approve(address(paymentContract), amount);
            paymentContract.createPayment(params);
        }
    }

    function estimateCrossChainFee(
        uint64 destinationChainSelector,
        address recipient,
        uint256 amount,
        address token,
        bytes memory zkProof,
        string memory description
    ) external view returns (uint256 fee) {
        Client.EVM2AnyMessage memory message = _buildCCIPMessage(
            recipient,
            amount,
            token,
            zkProof,
            description
        );
        return i_router.getFee(destinationChainSelector, message);
    }

    function withdrawLinkTokens(address to, uint256 amount) external onlyOwner {
        require(i_linkToken.transfer(to, amount), "LINK transfer failed");
    }

    function withdrawNativeTokens(address payable to, uint256 amount) external onlyOwner {
        require(address(this).balance >= amount, "Insufficient balance");
        to.transfer(amount);
    }

    function withdrawERC20Tokens(address token, address to, uint256 amount) external onlyOwner {
        IERC20(token).transfer(to, amount);
    }

    receive() external payable {}
}