# ProofPay Multi-Chain Deployment Guide

Complete guide for deploying ProofPay smart contracts across 9 blockchains (5 EVM + 4 Cosmos chains).

## 📋 Pre-Deployment Checklist

### 1. Environment Setup

#### Required Software
- [ ] Node.js (>=18.0.0) - `node --version`
- [ ] npm (>=9.0.0) - `npm --version`  
- [ ] Rust & Cargo - `cargo --version`
- [ ] Git - `git --version`
- [ ] Docker (optional, for CosmWasm optimization) - `docker --version`

#### Repository Setup
```bash
# Clone and setup
git clone https://github.com/your-org/proofpay-contract
cd proofpay-contract

# Install dependencies
npm run install:all

# Verify builds work
npm run build:all
```

### 2. Create Deployer Wallet

#### Generate New Wallet (Recommended)
```bash
# Using MetaMask or any wallet software
# Create new wallet specifically for deployment
# Export private key (keep secure!)
```

#### Or Use Existing Wallet
- Ensure it has sufficient funds on all target chains
- Export private key securely

### 3. Obtain API Keys

#### RPC Providers (Choose one)

**Option A: Alchemy (Recommended)**
1. Sign up at [alchemy.com](https://alchemy.com)
2. Create apps for:
   - Ethereum Mainnet
   - Ethereum Sepolia (testnet)
   - Polygon Mainnet
   - Arbitrum One
3. Copy API keys from dashboard

**Option B: Infura**
1. Sign up at [infura.io](https://infura.io)
2. Create project
3. Enable networks: Ethereum, Polygon, Arbitrum
4. Copy project ID

**Option C: QuickNode**
1. Sign up at [quicknode.com](https://quicknode.com)
2. Create endpoints for each chain
3. Copy RPC URLs

#### Block Explorer API Keys (Optional - for verification)

**Etherscan Family:**
1. **Etherscan**: [etherscan.io/apis](https://etherscan.io/apis)
2. **Polygonscan**: [polygonscan.com/apis](https://polygonscan.com/apis)
3. **BscScan**: [bscscan.com/apis](https://bscscan.com/apis)
4. **Arbiscan**: [arbiscan.io/apis](https://arbiscan.io/apis)

Sign up for each, verify email, generate API key.

### 4. Fund Deployer Wallet

#### Required Native Tokens

| Chain | Token | Amount Needed | USD (~) | Where to Buy |
|-------|-------|---------------|---------|--------------|
| Ethereum | ETH | 0.2 - 0.5 ETH | $400-1000 | Coinbase, Binance |
| Sepolia | SepoliaETH | 1 ETH | Free | [Faucet](https://sepoliafaucet.com) |
| Polygon | MATIC | 50 - 100 MATIC | $40-80 | Coinbase, Binance |
| BSC | BNB | 0.1 - 0.5 BNB | $25-125 | Coinbase, Binance |
| Arbitrum | ETH | 0.05 - 0.2 ETH | $100-400 | Bridge from Ethereum |

#### Getting Testnet Funds

**Sepolia ETH (Free):**
- [Alchemy Faucet](https://sepoliafaucet.com)
- [Chainlink Faucet](https://faucets.chain.link/sepolia)
- Requires social verification

**Polygon Mumbai (if using testnet):**
- [Polygon Faucet](https://faucet.polygon.technology)

#### Bridge to L2s

**Arbitrum:**
- Use [Arbitrum Bridge](https://bridge.arbitrum.io)
- Bridge ETH from Ethereum mainnet

**Polygon:**
- Use [Polygon Bridge](https://portal.polygon.technology)
- Or buy MATIC directly on exchanges

### 5. CosmWasm Chain Setup

#### Required CLI Tools

**XION:**
```bash
# Install xiond
wget https://github.com/burnt-labs/xion/releases/latest/download/xiond-linux-amd64
chmod +x xiond-linux-amd64
sudo mv xiond-linux-amd64 /usr/local/bin/xiond
```

**Osmosis:**
```bash
# Install osmosisd  
wget https://github.com/osmosis-labs/osmosis/releases/latest/download/osmosisd-linux-amd64
chmod +x osmosisd-linux-amd64
sudo mv osmosisd-linux-amd64 /usr/local/bin/osmosisd
```

**Neutron:**
```bash
# Install neutrond
wget https://github.com/neutron-org/neutron/releases/latest/download/neutrond-linux-amd64
chmod +x neutrond-linux-amd64 
sudo mv neutrond-linux-amd64 /usr/local/bin/neutrond
```

**Juno:**
```bash
# Install junod
wget https://github.com/CosmosContracts/juno/releases/latest/download/junod-linux-amd64
chmod +x junod-linux-amd64
sudo mv junod-linux-amd64 /usr/local/bin/junod
```

#### Create Cosmos Wallets
```bash
# Create wallet for each chain
xiond keys add deployer --keyring-backend test
osmosisd keys add deployer --keyring-backend test  
neutrond keys add deployer --keyring-backend test
junod keys add deployer --keyring-backend test

# Fund wallets through faucets or DEX
```

## ⚙️ Configuration

### 1. Environment Variables

```bash
# Copy template
cp .env.example .env

# Edit .env file
nano .env
```

**Required .env Contents:**
```bash
# Deployment
PRIVATE_KEY=0x1234567890abcdef1234567890abcdef12345678
VERIFY_CONTRACTS=true

# EVM RPC URLs  
ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
ETHEREUM_SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
POLYGON_RPC_URL=https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY
BSC_RPC_URL=https://bsc-dataseed1.binance.org/
ARBITRUM_RPC_URL=https://arb-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# Block Explorer API Keys (Optional)
ETHERSCAN_API_KEY=YOUR_ETHERSCAN_API_KEY
POLYGONSCAN_API_KEY=YOUR_POLYGONSCAN_API_KEY
BSCSCAN_API_KEY=YOUR_BSCSCAN_API_KEY
ARBISCAN_API_KEY=YOUR_ARBISCAN_API_KEY

# Cosmos Chain RPCs
XION_RPC_URL=https://rpc.xion-testnet-2.burnt.com:443
OSMOSIS_RPC_URL=https://rpc.osmosis.zone:443
NEUTRON_RPC_URL=https://rpc.neutron.org:443
JUNO_RPC_URL=https://rpc.juno.omniflix.co:443
```

### 2. Verify Configuration

```bash
# Test environment loads
npm run build:all

# Check wallet has funds
# (Use block explorers to verify balances)
```

## 🚀 Deployment Steps

### Phase 1: Testnet Deployment

#### Step 1: Deploy to Sepolia (Testnet)
```bash
# Deploy to Ethereum Sepolia first
npm run deploy:evm:sepolia

# Verify deployment worked
# Check deployment-summary.json for addresses
```

#### Step 2: Verify Testnet Deployment
```bash
# Verify contracts on Etherscan
cd packages/contracts-evm
npx hardhat verify --network sepolia CONTRACT_ADDRESS

# Test basic functionality
# - Register a user
# - Create a payment
# - Submit proof
```

#### Step 3: Fix Any Issues
- Review deployment logs in `logs/` directory
- Check for compilation errors
- Verify wallet has sufficient funds
- Debug any transaction failures

### Phase 2: Mainnet EVM Deployment

#### Step 1: Deploy EVM Mainnets
```bash
# Deploy to all EVM chains
npm run deploy:evm:polygon    # Start with cheapest gas
npm run deploy:evm:bsc        # Second cheapest
npm run deploy:evm:arbitrum   # Moderate gas
# npm run deploy:evm:ethereum # Most expensive - do last
```

#### Step 2: Verify Contracts
```bash
# Verify on all explorers
npm run verify:all
```

#### Step 3: Configure CCIP
```bash
# Update CCIP router approvals
# Set up cross-chain allowlists
# Fund CCIP contracts with LINK tokens
```

### Phase 3: CosmWasm Deployment

#### Step 1: Build Optimized WASM
```bash
cd packages/contracts-cosmwasm

# Optimize WASM binary
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13
```

#### Step 2: Deploy to Cosmos Chains

**XION:**
```bash
# Store code
xiond tx wasm store artifacts/proofpay_cosmwasm.wasm \
  --from deployer --gas auto --gas-adjustment 1.3 \
  --chain-id xion-testnet-1 --node https://rpc.xion-testnet-2.burnt.com:443

# Instantiate contract
CODE_ID=123  # From store response
xiond tx wasm instantiate $CODE_ID '{"admin":null}' \
  --label "ProofPay v1.0" --from deployer --admin deployer \
  --chain-id xion-testnet-1 --node https://rpc.xion-testnet-2.burnt.com:443
```

**Osmosis:**
```bash
# Similar process for each chain
osmosisd tx wasm store artifacts/proofpay_cosmwasm.wasm \
  --from deployer --gas auto --gas-adjustment 1.3 \
  --chain-id osmosis-1 --node https://rpc.osmosis.zone:443
```

**Neutron & Juno:**
```bash
# Repeat for remaining chains
neutrond tx wasm store artifacts/proofpay_cosmwasm.wasm --from deployer --chain-id neutron-1
junod tx wasm store artifacts/proofpay_cosmwasm.wasm --from deployer --chain-id juno-1
```

#### Step 3: Set Up IBC Channels
```bash
# Create IBC channels between chains
# Configure channel allowlists
# Test cross-chain payments
```

### Phase 4: Cross-Chain Configuration

#### Step 1: EVM CCIP Setup
```bash
# Set allowed destination chains
# Configure trusted senders
# Fund contracts with LINK tokens
```

#### Step 2: Cosmos IBC Setup  
```bash
# Establish IBC channels
# Configure channel permissions
# Test packet transmission
```

#### Step 3: End-to-End Testing
```bash
# Test cross-chain payments
# Verify proof systems work
# Check all chains can communicate
```

## ✅ Post-Deployment

### 1. Update Documentation
- [ ] Update README.md with contract addresses
- [ ] Create deployment report
- [ ] Document any issues encountered

### 2. Security Checks
- [ ] Verify all contracts on explorers
- [ ] Test emergency functions work
- [ ] Run security audit tools
- [ ] Test pause/unpause mechanisms

### 3. Integration
- [ ] Update frontend with new contract addresses
- [ ] Update SDK/API configurations  
- [ ] Test user flows end-to-end
- [ ] Update monitoring/alerts

### 4. Launch Preparation
- [ ] Prepare announcement materials
- [ ] Set up monitoring dashboards
- [ ] Create incident response plan
- [ ] Train support team

## 🚨 Troubleshooting

### Common Issues

**Deployment Fails:**
- Check wallet has sufficient funds
- Verify RPC URLs are working
- Check gas limit settings
- Review Hardhat configuration

**Verification Fails:**
- Wait 5-10 minutes after deployment
- Check constructor arguments match
- Verify API keys are correct
- Try manual verification on explorer

**CCIP Issues:**
- Ensure LINK tokens funded
- Check router addresses are correct
- Verify chain selectors match
- Test fee estimation

**CosmWasm Issues:**
- Check WASM binary is optimized
- Verify gas settings sufficient
- Ensure wallet funded on each chain
- Check CLI tool versions

### Getting Help

**Discord Communities:**
- Chainlink Discord (CCIP support)
- CosmWasm Discord (CosmWasm support)
- XION Discord (XION specific)

**Documentation:**
- [Chainlink CCIP Docs](https://docs.chain.link/ccip)
- [CosmWasm Docs](https://docs.cosmwasm.com)
- [Hardhat Docs](https://hardhat.org/docs)

**Emergency Contacts:**
- Deployment issues: Create GitHub issue
- Security concerns: Email security@proofpay.com

---

## 📊 Cost Estimates

| Phase | Chain | Est. Cost | Time |
|-------|-------|-----------|------|
| Testnet | Sepolia | $0 (free) | 30 min |
| Mainnet | Polygon | $5-15 | 1 hour |
| Mainnet | BSC | $2-8 | 1 hour |
| Mainnet | Arbitrum | $20-50 | 1 hour |
| Mainnet | Ethereum | $200-800 | 1 hour |
| CosmWasm | All 4 chains | $5-20 | 2 hours |
| **Total** | | **$232-893** | **6-7 hours** |

**Ready to deploy across all chains? Follow this guide step by step!** 🚀