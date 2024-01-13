import { Provider, WalletUnlocked, Wallet } from 'fuels';
import { NftAbi__factory } from './src/contracts/factories/NftAbi__factory';
import fs from 'fs/promises';

(async function() {
  // const provider = Provider.create('http://localhost:4000/graphql');
  // const wallet = Wallet.fromPrivateKey('0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb', provider);

  // const nftBin = await fs.readFile('../nft/out/debug/nft.bin');
  // const nft = await NftAbi__factory.deployContract(nftBin, wallet);
  console.log('nft');
})()
