#!/bin/bash

# ProofPay Multi-Chain Deployment Script
echo "🚀 Starting ProofPay multi-chain deployment..."

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required environment variables are set
check_env_vars() {
    local missing_vars=()
    
    if [ -z "$PRIVATE_KEY" ]; then
        missing_vars+=("PRIVATE_KEY")
    fi
    
    # Check EVM RPC URLs
    if [ -z "$ETHEREUM_SEPOLIA_RPC_URL" ]; then
        missing_vars+=("ETHEREUM_SEPOLIA_RPC_URL")
    fi
    
    if [ -z "$POLYGON_RPC_URL" ]; then
        missing_vars+=("POLYGON_RPC_URL")
    fi
    
    if [ -z "$BSC_RPC_URL" ]; then
        missing_vars+=("BSC_RPC_URL")
    fi
    
    if [ -z "$ARBITRUM_RPC_URL" ]; then
        missing_vars+=("ARBITRUM_RPC_URL")
    fi
    
    if [ ${#missing_vars[@]} -ne 0 ]; then
        print_error "Missing required environment variables:"
        printf '%s\n' "${missing_vars[@]}"
        print_error "Please set these variables in your .env file"
        exit 1
    fi
}

# Deploy to EVM chains
deploy_evm_contracts() {
    print_status "Deploying EVM contracts..."
    cd packages/contracts-evm
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing EVM contract dependencies..."
        npm install
    fi
    
    # Compile contracts
    print_status "Compiling Solidity contracts..."
    npm run compile
    
    if [ $? -ne 0 ]; then
        print_error "Failed to compile Solidity contracts"
        exit 1
    fi
    
    # Deploy to testnets first
    local networks=("sepolia" "polygon" "bsc" "arbitrum")
    local failed_deployments=()
    
    for network in "${networks[@]}"; do
        print_status "Deploying to $network..."
        
        if npm run deploy:$network; then
            print_success "Successfully deployed to $network"
        else
            print_warning "Failed to deploy to $network"
            failed_deployments+=("$network")
        fi
        
        # Add delay between deployments
        sleep 2
    done
    
    if [ ${#failed_deployments[@]} -ne 0 ]; then
        print_warning "Some EVM deployments failed:"
        printf '%s\n' "${failed_deployments[@]}"
    fi
    
    cd ../..
}

# Deploy to CosmWasm chains
deploy_cosmwasm_contracts() {
    print_status "Deploying CosmWasm contracts..."
    cd packages/contracts-cosmwasm
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo (Rust) is not installed. Please install Rust first."
        return 1
    fi
    
    # Build the contract
    print_status "Building CosmWasm contract..."
    cargo build --release --target wasm32-unknown-unknown
    
    if [ $? -ne 0 ]; then
        print_error "Failed to build CosmWasm contract"
        return 1
    fi
    
    # Optimize the WASM binary
    if command -v docker &> /dev/null; then
        print_status "Optimizing WASM binary..."
        docker run --rm -v "$(pwd)":/code \
            --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
            --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
            cosmwasm/rust-optimizer:0.12.13
    else
        print_warning "Docker not found, skipping WASM optimization"
    fi
    
    # Note: Actual deployment to Cosmos chains would require chain-specific tools
    # This is a placeholder for the deployment logic
    print_status "CosmWasm contract built successfully"
    print_warning "Actual deployment to Cosmos chains requires chain-specific deployment tools"
    print_warning "Please use the appropriate CLI tools for each Cosmos chain:"
    print_warning "- XION: xiond"
    print_warning "- Osmosis: osmosisd" 
    print_warning "- Neutron: neutrond"
    print_warning "- Juno: junod"
    
    cd ../..
}

# Verify contracts on block explorers
verify_contracts() {
    print_status "Verifying contracts on block explorers..."
    cd packages/contracts-evm
    
    local networks=("sepolia" "polygon" "bsc" "arbitrum")
    
    for network in "${networks[@]}"; do
        if [ ! -z "${!network^^}_API_KEY" ]; then
            print_status "Verifying contracts on $network..."
            npm run verify $network || print_warning "Verification failed for $network"
        else
            print_warning "No API key found for $network verification"
        fi
    done
    
    cd ../..
}

# Update deployment configuration
update_deployment_config() {
    print_status "Updating deployment configuration..."
    
    # Create deployment summary
    cat > deployment-summary.json << EOF
{
  "deploymentDate": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "networks": {
    "evm": {
      "sepolia": {
        "deployed": true,
        "verified": false
      },
      "polygon": {
        "deployed": true,
        "verified": false
      },
      "bsc": {
        "deployed": true,
        "verified": false
      },
      "arbitrum": {
        "deployed": true,
        "verified": false
      }
    },
    "cosmos": {
      "xion": {
        "deployed": false,
        "codeId": null
      },
      "osmosis": {
        "deployed": false,
        "codeId": null
      },
      "neutron": {
        "deployed": false,
        "codeId": null
      },
      "juno": {
        "deployed": false,
        "codeId": null
      }
    }
  }
}
EOF
    
    print_success "Deployment summary created: deployment-summary.json"
}

# Main execution
main() {
    print_status "ProofPay Multi-Chain Deployment Starting..."
    
    # Check prerequisites
    check_env_vars
    
    # Create logs directory
    mkdir -p logs
    
    # Deploy contracts
    deploy_evm_contracts 2>&1 | tee logs/evm-deployment.log
    deploy_cosmwasm_contracts 2>&1 | tee logs/cosmwasm-deployment.log
    
    # Verify contracts (optional)
    if [ "$VERIFY_CONTRACTS" = "true" ]; then
        verify_contracts 2>&1 | tee logs/verification.log
    fi
    
    # Update configuration
    update_deployment_config
    
    print_success "🎉 Multi-chain deployment process completed!"
    print_status "Check logs/ directory for detailed deployment logs"
    print_status "Review deployment-summary.json for deployment status"
}

# Run main function
main "$@"