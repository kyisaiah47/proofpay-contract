import { EVMChainInfo, CosmosChainInfo } from './types';

export const SUPPORTED_EVM_CHAINS: Record<string, EVMChainInfo> = {
  ETHEREUM: {
    chainId: 1,
    name: 'Ethereum Mainnet',
    rpcUrl: process.env.ETHEREUM_RPC_URL,
    ccipChainSelector: '5009297550715157269',
    deploymentAddress: '',
  },
  SEPOLIA: {
    chainId: 11155111,
    name: 'Ethereum Sepolia',
    rpcUrl: process.env.ETHEREUM_SEPOLIA_RPC_URL,
    ccipChainSelector: '16015286601757825753',
    deploymentAddress: '',
  },
  POLYGON: {
    chainId: 137,
    name: 'Polygon Mainnet',
    rpcUrl: process.env.POLYGON_RPC_URL,
    ccipChainSelector: '4051577828743386545',
    deploymentAddress: '',
  },
  BSC: {
    chainId: 56,
    name: 'BNB Smart Chain',
    rpcUrl: process.env.BSC_RPC_URL,
    ccipChainSelector: '11344663589394136015',
    deploymentAddress: '',
  },
  ARBITRUM: {
    chainId: 42161,
    name: 'Arbitrum One',
    rpcUrl: process.env.ARBITRUM_RPC_URL,
    ccipChainSelector: '4949039107694359620',
    deploymentAddress: '',
  },
};

export const SUPPORTED_COSMOS_CHAINS: Record<string, CosmosChainInfo> = {
  XION: {
    chainId: 'xion-testnet-1',
    name: 'XION Testnet',
    rpcUrl: 'https://rpc.xion-testnet-2.burnt.com:443',
    addressPrefix: 'xion',
    deploymentAddress: '',
  },
  OSMOSIS: {
    chainId: 'osmosis-1',
    name: 'Osmosis',
    rpcUrl: 'https://rpc.osmosis.zone:443',
    addressPrefix: 'osmo',
    deploymentAddress: '',
  },
  NEUTRON: {
    chainId: 'neutron-1',
    name: 'Neutron',
    rpcUrl: 'https://rpc.neutron.org:443',
    addressPrefix: 'neutron',
    deploymentAddress: '',
  },
  JUNO: {
    chainId: 'juno-1',
    name: 'Juno',
    rpcUrl: 'https://rpc.juno.omniflix.co:443',
    addressPrefix: 'juno',
    deploymentAddress: '',
  },
};

export const CONTRACT_CONSTANTS = {
  MAX_DESCRIPTION_LENGTH: 500,
  MAX_PROOF_SIZE: 10000,
  MAX_USERNAME_LENGTH: 32,
  MAX_DISPUTE_REASON_LENGTH: 200,
  IBC_VERSION: 'proofpay-1',
  CCIP_GAS_LIMIT: 500_000,
} as const;

export const CHAINLINK_CCIP_ROUTERS = {
  [SUPPORTED_EVM_CHAINS.ETHEREUM.chainId]: '0x80226fc0Ee2b096224EeAc085Bb9a8cba1146f7D',
  [SUPPORTED_EVM_CHAINS.SEPOLIA.chainId]: '0x0BF3dE8c5D3e8A2B34D2BEeB17ABfCeBaf363A59',
  [SUPPORTED_EVM_CHAINS.POLYGON.chainId]: '0x3C3D92629A02a8D95D5CB9650fe49C3544f69B43',
  [SUPPORTED_EVM_CHAINS.BSC.chainId]: '0x34B03Cb9086d7D758AC55af71584F81A598759FE',
  [SUPPORTED_EVM_CHAINS.ARBITRUM.chainId]: '0x141fa059441E0ca23ce184B6A78bafD2A517DdE8',
} as const;

export const CHAINLINK_LINK_TOKENS = {
  [SUPPORTED_EVM_CHAINS.ETHEREUM.chainId]: '0x514910771AF9Ca656af840dff83E8264EcF986CA',
  [SUPPORTED_EVM_CHAINS.SEPOLIA.chainId]: '0x779877A7B0D9E8603169DdbD7836e478b4624789',
  [SUPPORTED_EVM_CHAINS.POLYGON.chainId]: '0xb0897686c545045aFc77CF20eC7A532E3120E0F1',
  [SUPPORTED_EVM_CHAINS.BSC.chainId]: '0x404460C6A5EdE2D891e8297795264fDe62ADBB75',
  [SUPPORTED_EVM_CHAINS.ARBITRUM.chainId]: '0xf97f4df75117a78c1A5a0DBb814Af92458539FB4',
} as const;

export const GAS_LIMITS = {
  USER_REGISTRATION: 100_000,
  PAYMENT_CREATION: 200_000,
  PAYMENT_COMPLETION: 150_000,
  CROSS_CHAIN_SEND: 500_000,
  PROOF_SUBMISSION: 120_000,
} as const;

export const ERROR_MESSAGES = {
  USER_NOT_REGISTERED: 'User not registered',
  USERNAME_TAKEN: 'Username already taken',
  PAYMENT_NOT_FOUND: 'Payment not found',
  INSUFFICIENT_FUNDS: 'Insufficient funds',
  UNAUTHORIZED: 'Unauthorized',
  INVALID_AMOUNT: 'Invalid amount',
  INVALID_PROOF: 'Invalid proof',
  CHAIN_NOT_SUPPORTED: 'Chain not supported',
  CCIP_FEE_CALCULATION_FAILED: 'CCIP fee calculation failed',
  IBC_CHANNEL_NOT_FOUND: 'IBC channel not found',
} as const;