import { Provider, WalletUnlocked, Wallet } from 'fuels';
import { NftAbi__factory } from '@/contracts/factories/NftAbi__factory';
import nftBin from '@/contracts/NftAbi.hex';
import fs from 'fs/promises';

export async function GET(request: Request) {
  try {
    const provider = await Provider.create('http://localhost:4000/graphql');
    const wallet = Wallet.fromPrivateKey('0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb', provider);

    const nft = await NftAbi__factory.deployContract(nftBin, wallet);
    console.log(nft);

    return Response.json({})
  } catch (e) {
    console.error(e);
    return Response.json({ error: e })
  }
}
