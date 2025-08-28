export enum PaymentStatus {
  Pending = 'pending',
  Completed = 'completed',
  Disputed = 'disputed',
  Cancelled = 'cancelled'
}

export enum ProofType {
  None = 'none',
  Text = 'text',
  Photo = 'photo',
  zkTLS = 'zktls',
  Hybrid = 'hybrid'
}

export interface User {
  username: string;
  isRegistered: boolean;
  authorizedAddresses: string[];
  createdAt: number;
}

export interface Payment {
  id: string;
  sender: string;
  recipient: string;
  amount: string;
  token?: string;
  status: PaymentStatus;
  proofType?: ProofType;
  proofData?: string;
  description?: string;
  createdAt: number;
  completedAt?: number;
  requiresProof: boolean;
}

export interface PaymentParams {
  recipient: string;
  amount: string;
  token?: string;
  proofType?: ProofType;
  description?: string;
  requiresProof: boolean;
}

export interface CrossChainPayment {
  destinationChain: string;
  recipient: string;
  amount: string;
  token?: string;
  zkProof?: string;
  description?: string;
}

export interface Stats {
  totalUsers: number;
  totalPayments: number;
  totalVolume: string;
}

export interface ChainInfo {
  chainId: string | number;
  name: string;
  rpcUrl?: string;
  deploymentAddress?: string;
}

export interface EVMChainInfo extends ChainInfo {
  chainId: number;
  ccipChainSelector?: string;
}

export interface CosmosChainInfo extends ChainInfo {
  chainId: string;
  addressPrefix: string;
}

export interface ContractAddresses {
  userContract?: string;
  paymentContract?: string;
  ccipContract?: string;
  mainContract?: string;
}

export interface DeploymentConfig {
  network: string;
  chainInfo: ChainInfo;
  contracts: ContractAddresses;
  verification?: {
    apiKey?: string;
    url?: string;
  };
}