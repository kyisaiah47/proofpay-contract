# ProofPay Multi-Chain Smart Contracts

ProofPay's multi-chain smart contract system supporting both EVM (Ethereum, Polygon, BSC, Arbitrum) and Cosmos (XION, Osmosis, Neutron, Juno) ecosystems with cross-chain payment capabilities.

## 🏗️ Architecture Overview

```
packages/
├── contracts-evm/          # Solidity contracts for EVM chains
│   ├── contracts/
│   │   ├── ProofPay.sol           # Main contract
│   │   ├── ProofPayUsers.sol      # User management
│   │   ├── ProofPayments.sol      # Payment processing
│   │   └── ProofPayCCIP.sol       # Cross-chain integration
│   ├── deploy/
│   ├── test/
│   └── hardhat.config.ts
├── contracts-cosmwasm/     # CosmWasm contracts for Cosmos chains
│   ├── src/
│   │   ├── contract.rs            # Main contract logic
│   │   ├── msg.rs                 # Message definitions
│   │   ├── state.rs               # State management
│   │   └── ibc.rs                 # IBC integration
│   └── Cargo.toml
└── shared/                 # Common types and constants
    ├── types.ts
    ├── constants.ts
    └── index.ts
```

## 🚀 Quick Start

### Prerequisites

- Node.js (>=18.0.0)
- Rust and Cargo
- Docker (for CosmWasm optimization)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/proofpay-contract
cd proofpay-contract

# Install all dependencies
npm run install:all
```

### Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit .env with your configuration
# - Private keys
# - RPC URLs
# - API keys for verification
```

### Build All Contracts

```bash
# Build EVM and CosmWasm contracts
npm run build:all
```

### Deploy to All Chains

```bash
# Deploy to all supported chains
npm run deploy:all
```

## 🔗 Supported Chains

### EVM Chains (with Chainlink CCIP)
- **Ethereum Mainnet/Sepolia**
- **Polygon** 
- **BNB Smart Chain**
- **Arbitrum One**

### Cosmos Chains (with IBC)
- **XION** - Primary testnet chain
- **Osmosis** - DEX and liquidity hub
- **Neutron** - Smart contracts platform
- **Juno** - Interoperable smart contracts

## 🛠️ Core Features

### User Management
- Decentralized user registration with usernames
- Multi-address authorization system
- Cross-chain identity consistency

### Payment Processing
- Native and token payments
- Proof-based verification system
- Multi-step payment flows
- Dispute resolution

### Cross-Chain Integration
- **CCIP (EVM)**: Seamless cross-chain payments between EVM chains
- **IBC (Cosmos)**: Inter-blockchain communication for Cosmos ecosystem
- Unified payment experience across all chains

### Proof Systems
- Text-based proofs
- Photo verification
- zkTLS integration
- Hybrid proof combinations

## 📋 Available Scripts

### Development
```bash
npm run build:all          # Build all contracts
npm run test:all           # Run all tests
npm run clean              # Clean build artifacts
```

### Deployment
```bash
npm run deploy:all         # Deploy to all chains
npm run deploy:evm:sepolia # Deploy to specific EVM chain
npm run verify:all         # Verify contracts on explorers
```

### Chain-Specific
```bash
# EVM chains
npm run deploy:evm:polygon
npm run deploy:evm:bsc
npm run deploy:evm:arbitrum

# CosmWasm chains require chain-specific tools
# See deployment documentation
```

## 🔧 Configuration

### Chain Configuration

Chain configurations are defined in `packages/shared/constants.ts`:

```typescript
export const SUPPORTED_EVM_CHAINS = {
  ETHEREUM: {
    chainId: 1,
    ccipChainSelector: '5009297550715157269',
    // ...
  },
  // ...
};

export const SUPPORTED_COSMOS_CHAINS = {
  XION: {
    chainId: 'xion-testnet-1',
    addressPrefix: 'xion',
    // ...
  },
  // ...
};
```

### Contract Addresses

After deployment, contract addresses are automatically updated in:
- `packages/contracts-evm/deployments/`
- `deployment-summary.json`

## 🧪 Testing

### EVM Contracts
```bash
cd packages/contracts-evm
npm test
```

### CosmWasm Contracts
```bash
cd packages/contracts-cosmwasm
cargo test
```

## 📝 Contract Interaction Examples

### Register User
```solidity
// Solidity
proofPay.registerUser("alice");
```

```rust
// CosmWasm
ExecuteMsg::RegisterUser { username: "alice".to_string() }
```

### Create Payment
```solidity
// Solidity
PaymentParams memory params = PaymentParams({
    recipient: recipientAddress,
    amount: 1000000, // 1 USDC (6 decimals)
    token: usdcAddress,
    proofType: ProofType.Text,
    description: "Payment for services",
    requiresProof: true
});
proofPay.createPayment{value: 0}(params);
```

### Cross-Chain Payment (CCIP)
```solidity
CrossChainPayment memory payment = CrossChainPayment({
    destinationChain: 4051577828743386545, // Polygon
    recipient: recipientAddress,
    amount: 1000000,
    token: usdcAddress,
    zkProof: proofData,
    description: "Cross-chain payment"
});
proofPayCCIP.sendCrossChainPayment{value: msg.value}(payment);
```

## 🛡️ Security Features

- **Reentrancy Protection**: All external calls protected
- **Access Control**: Role-based permissions
- **Input Validation**: Comprehensive parameter validation
- **Cross-Chain Security**: Source validation for CCIP/IBC
- **Rate Limiting**: Per-user and global limits
- **Emergency Controls**: Circuit breakers implemented

## 📊 Gas Optimization

### Optimized Storage Patterns
```solidity
struct PackedPayment {
    address sender;      // 20 bytes
    address recipient;   // 20 bytes  
    uint96 amount;      // 12 bytes (packed)
    uint32 createdAt;   // 4 bytes
    PaymentStatus status; // 1 byte
    ProofType proofType; // 1 byte
}
```

### Gas Limits
- User Registration: ~100k gas
- Payment Creation: ~200k gas
- Cross-chain Send: ~500k gas

## 🔗 Cross-Chain Cost Estimates

| Route | Estimated Fee | Speed | Security |
|-------|--------------|--------|----------|
| Ethereum → Polygon | $5-15 | 5-10 min | High |
| Polygon → BSC | $3-8 | 3-7 min | High |
| XION → Osmosis | $0.1-0.5 | 1-3 min | High |
| Osmosis → Neutron | $0.05-0.2 | 30s-2min | High |

## 🚨 Emergency Procedures

### Circuit Breakers
All contracts include emergency pause functionality for critical issues.

### Upgrade Patterns
Contracts use proxy patterns for safe upgrades (where applicable).

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📞 Support

For support and questions:
- Create an issue in this repository
- Join our Discord community
- Email: support@proofpay.com

---

**Ready to build the future of verified payments across all blockchains!** 🚀
