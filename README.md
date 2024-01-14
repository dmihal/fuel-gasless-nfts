# Fuel gasless NFT

This repo is an example application built for Fuel, which allows users to mint and transfer NFTs without needing to pay any gas fees. This allows users to use applications without needing to first purchase ETH on an exchange, on-ramp, etc.

The application allows a third-party to "sponsor" these transactions, by putting some ETH in a predicate that enforces the conditions by which it can be used in a transaction.

## Whitelisting

Since sponsored transactions present the potential for a user to "grief" the sponsor by sending spam transactions (such as sending an NFT back-and-forth), there needs to be some system to add and remove accounts from a whitelist.

This whitelist is implemented in two ways, both on-chain and off-chain. Off-chain approval is provided by sending a transaction to a server endpoint, which will sign the transaction if it comes from a whitelisted address. Additionally, the whitelist can be stored on-chain in the form of a packet (a non-financial UTXO representing state) which represents a single address being whitelisted. This packet can be burned at any time by the issuer, effectively removing the address from the whitelist

## Technical components

* **NFT smart contract:** a simple SRC-20 contract that mints NFTs and provides metadata.
* **Gas predicate:** a predicate that holds ETH for gas, and enforces the conditions of the transaction. These conditions include:
  * Ensuring the account is whitelisted, either via signature verification or packet verification.
  * Checking that the transaction uses the NFT script (for minting), or no script (for transfers).
  * Checking all inputs and outputs to ensure that they only consist of ETH (for gas, with change returned to the predicate), NFTs (for transfers), and whitelist packets, as well as relevant smart contracts.
* **NFT script:** a basic script that allows a single NFT to be minted, as well as optionally allowing a whitelist packet to be minted.
* **Packet minter contract:** simple contract that mints a UTXO to the packet predicate. Uses the same signature verification as the gas predicate.
* **Packet predicate:** simple predicate for holding the packets. Packets can be included in any transaction, as long as the packet is returned back to the predicate (essentially allowing for read-only UTXOs). Alternatively, an administrator can sign the transaction, allowing for packets to be removed from the predicate (to remove from the whitelist).

## Operations

### Minting an NFT (with signature)

* The user connects their wallet to the application (or uses an embedded, burner-style wallet).
* The user clicks mint, and the frontend generates a mint transaction (using the NFT script and ETH from the gas predicate).
* The frontend sends the mint transaction to a server endpoint. The server validates that the recipient is not on a blacklist, then generates a signature and returns it to the user.
* The frontend attaches the signature to the transaction, and submits it to be included on-chain.
* The transaction mints an NFT to the user's address

### Minting an NFT (with signature, generating packet)

Same as the previous operation, but the packet-minter contract will be called and a packet is minted to the packet predicate address

### Transferring an NFT (with signature)

* User uses the frontend do prepare a transfer, the frontend generates a transaction with no script, just the NFT as an input & output, plus ETH from the gas predicate
* The frontend sends the transaction to a server endpoint. The server validates that the sender & recipient are not on a blacklist, then generates a signature and returns it to the user.
* The frontend attaches the signature to the transaction, and submits it to be included on-chain.
* The transaction transfers the NFT

### Transferring an NFT (with a packet)

* User uses the frontend do prepare a transfer, the frontend generates a transaction with no script, just the NFT as an input & output, ETH from the gas predicate, and the packet as an input & output.
* The frontend attaches the signature to the transaction, and submits it to be included on-chain.
* During execution, the gas predicate validates the user's address from the packet.
* The transaction transfers the NFT.

### Revoking a whitelist packet

* The admin creates a transaction with a given packet as an input, but not as an output, effectively burning it
* The admin signs the transaction with the admin wallet. Gas is paid out of their own wallet.
* The packet predicate validates the signature.

## Limitations

If used at scale, users may face concurrency issues, since multiple users may try to spend the same ETH coins simultaneously.

This issue can be reduced by keeping many smaller coins in the gas predicate, having the frontend select coins at random, and automatically re-submitting in the case of concurrency issues. However, fully solving this issue will require a more intent-style transaction model, allowing the available UTXOs to be selected by block builders instead of end users.

## Potential improvements

* Use a packet to represent authorized signers (instead of having the signer as an immutable configurable in the gas predicate)
