// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from 'next'
import { Wallet, Provider } from 'fuels'
import { hash } from '@fuel-ts/hasher'

type Data = {
  signature: string
}

type Error = {
  error: string
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data | Error>
) {
  try {
    if (req.method !== 'POST') {
      throw new Error('Must send POST request')
    }
    if (!process.env.SERVER_KEY) {
      throw new Error('Must set key')
    }

    const { txId } = req.body
    if (!txId || /0x[0-9a-fA-F]{64}/.test(txId)) {
      throw new Error('Invalid txId')
    }

    const provider = await Provider.create('http://127.0.0.1:4000/graphql')
    const wallet = Wallet.fromPrivateKey(process.env.SERVER_KEY, provider)
    const signature = wallet.signer().sign(hash(txId))

    res.status(200).json({ signature })
  } catch (e: any) {
    console.error(e)
    res.status(401).json({ error: e.message })
  }
}
