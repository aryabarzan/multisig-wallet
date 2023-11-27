
# Multisig Wallet for Concordium blockchain

A smart contract that requires 3 different accounts’ signatures to authorize some CCD (the local coin of Concordium) transfers on [Concordium](https://www.concordium.com/) blockchain. This contract is written in Rust.

##About Concordium
> [Concordium](https://www.concordium.com/) is a science-backed blockchain that emphasizes decentralized identity. It stands out as the only Layer 1 (L1) blockchain offering a built-in, self-sovereign identity feature. This means that each user's real-world identity is authenticated, but their privacy is still maintained. The design of the Concordium blockchain is aimed at being fast, secure, and cost-effective.

## Documentation

The contract is in the `./src/lib.rs`. In the init phase, three accounts are identified as owners. In order to execute a transfer request, all three owners must have previously supported (confirmed) it.

In order to transfer some CCD, first one of owners should submit his/her transfer request to this contract. Submitting a transaction by an owner means supporting it. So, the two other owners should support (confirm) that request. After getting support from all three owners, one of owners can execute the request.

The main functions is listed bellow:

| functions | Description |
| --- | --- |
| `contract_init` | Create the initial state |
| `contract_receive_deposit` | It is called when someone wants to transfer some CCD to this contract|
| `contract_receive_submit_transfer_request` | submit a transfer request by an owner  |
| `contract_receive_support_transfer_request` | One of owners support the request, if have not done before  |
| `contract_receive_not_support_transfer_request` | it is for revoking support on a request  |
|`contract_receive_execute_transfer_request`| It execute a transfer request, if all owners previously supported it. After successful execution, it removes from requests list|
|`contract_receive_view_transfer_request`| It gives a request id and return the associated request|