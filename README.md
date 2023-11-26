
# Multisig Wallet for Concordium blockchain

A smart contract that requires 3 different accounts’ signatures to authorize some CCD (the local coin of Concordium) transfers on [Concordium](https://www.concordium.com/) blockchain. This contratc is written in Rust.


## Documentation

The contratc is in the `./src/lib.rs`. This contratc has three owners that all those should aggree on each transfer.

In order to transfer some CCD, first one of owners should submit his/her transfer request to this contract. Then two other owners should support (confirm) that request. If all three owners, one of owners can execute the request  
The main functions is listed bellow:

| function | Description |
| --- | --- |
| `contract_init` | Create the initial state |
| `contract_receive_deposit` | It is called when someone wants to transfer some CCD to this contract|
| `contract_receive_submit_transfer_request` | When one of the owners want to  |