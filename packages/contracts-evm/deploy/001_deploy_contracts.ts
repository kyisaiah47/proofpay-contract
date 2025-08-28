import { HardhatRuntimeEnvironment } from 'hardhat/types';
import { DeployFunction } from 'hardhat-deploy/types';
import { CHAINLINK_CCIP_ROUTERS, CHAINLINK_LINK_TOKENS } from '@proofpay/shared';

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts, network } = hre;
  const { deploy, log } = deployments;
  const { deployer } = await getNamedAccounts();

  const chainId = network.config.chainId;
  if (!chainId) {
    throw new Error('Chain ID not found');
  }

  log(`Deploying ProofPay contracts to ${network.name} (chainId: ${chainId})`);
  log(`Deployer: ${deployer}`);

  // Deploy ProofPayUsers contract
  const proofPayUsers = await deploy('ProofPayUsers', {
    from: deployer,
    args: [],
    log: true,
    autoMine: true,
  });

  log(`ProofPayUsers deployed at: ${proofPayUsers.address}`);

  // Deploy ProofPayments contract
  const proofPayments = await deploy('ProofPayments', {
    from: deployer,
    args: [proofPayUsers.address],
    log: true,
    autoMine: true,
  });

  log(`ProofPayments deployed at: ${proofPayments.address}`);

  // Deploy main ProofPay contract
  const proofPay = await deploy('ProofPay', {
    from: deployer,
    args: [],
    log: true,
    autoMine: true,
  });

  log(`ProofPay deployed at: ${proofPay.address}`);

  // Deploy CCIP contract if supported chain
  const ccipRouter = CHAINLINK_CCIP_ROUTERS[chainId as keyof typeof CHAINLINK_CCIP_ROUTERS];
  const linkToken = CHAINLINK_LINK_TOKENS[chainId as keyof typeof CHAINLINK_LINK_TOKENS];

  if (ccipRouter && linkToken) {
    const proofPayCCIP = await deploy('ProofPayCCIP', {
      from: deployer,
      args: [ccipRouter, linkToken, proofPayments.address],
      log: true,
      autoMine: true,
    });

    log(`ProofPayCCIP deployed at: ${proofPayCCIP.address}`);
  } else {
    log(`CCIP not supported on ${network.name}, skipping ProofPayCCIP deployment`);
  }

  // Save deployment addresses
  const deploymentInfo = {
    network: network.name,
    chainId,
    deployer,
    contracts: {
      ProofPayUsers: proofPayUsers.address,
      ProofPayments: proofPayments.address,
      ProofPay: proofPay.address,
      ...(ccipRouter && linkToken && { ProofPayCCIP: proofPayCCIP.address }),
    },
    timestamp: new Date().toISOString(),
  };

  log('Deployment completed successfully!');
  log('Contract addresses:', deploymentInfo.contracts);
};

func.tags = ['ProofPay'];
export default func;