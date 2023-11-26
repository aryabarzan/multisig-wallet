#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_smart_contract_testing::*;
use concordium_std::collections::*;
use concordium_std::*;
use multisig_wallet::*;

#[concordium_cfg_test]
#[allow(deprecated)]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    /// Initialise contract with account owners
    fn test_init() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);

        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);
        owners.insert(account3);

        let parameter = InitParams { owners };
        let parameter_bytes = to_bytes(&parameter);

        let mut ctx = TestInitContext::empty();
        ctx.set_parameter(&parameter_bytes);

        let amount = Amount::from_micro_ccd(0);
        let mut state_builder = TestStateBuilder::new();
        // call the init function
        let out = contract_init(&ctx, &mut state_builder, amount);

        // and inspect the result.
        let state = match out {
            Ok(s) => s,
            Err(_) => fail!("Contract initialization failed."),
        };
        claim!(
            state.owners.contains(&account1),
            "Should contain the first owner"
        );
        claim!(
            state.owners.contains(&account2),
            "Should contain the second owner"
        );
        claim!(
            state.owners.contains(&account3),
            "Should contain the third owner"
        );
        claim_eq!(
            state.owners.len(),
            TRANSFER_AGREEMENT_THRESHOLD,
            "Should not contain more account owners"
        );
        // and make sure the correct logs were produced.
        claim_eq!(
            state.requests.iter().count(),
            0,
            "No transfer request at initialisation"
        );
    }

    #[concordium_test]
    /// Creates the transfer request
    ///
    /// - Mutates the state with the request
    /// - Only have the sender support the request at this point
    /// - last_request_id should be incremented
    fn test_receive_submit_transfer_request() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);
        let target_account = AccountAddress([4u8; 32]);
        let transfer_amount = Amount::from_micro_ccd(50);

        // Setup state
        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);
        owners.insert(account3);

        let mut state_builder = TestStateBuilder::new();
        let state = State {
            owners,
            last_request_id: 0,
            requests: state_builder.new_map(),
        };
        let mut host = TestHost::new(state, state_builder);
        host.set_self_balance(Amount::from_micro_ccd(0));

        // Create a request to transfer 50 CCD to target_account
        let parameter = SubmitParams {
            target_account,
            transfer_amount,
        };
        let parameter_bytes = to_bytes(&parameter);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_parameter(&parameter_bytes);
        ctx.set_sender(Address::Account(account1));
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(0));

        // Execution
        let res = contract_receive_submit_transfer_request(&ctx, &mut host);
        claim!(
            res.is_ok(),
            "Contract receive failed, but it should not have."
        );

        let request_id = res.unwrap();
        claim_eq!(request_id, 1, "`last_request_id` does not increment");

        let request = host.state().requests.get(&request_id).unwrap();
        claim_eq!(
            request.supporters.len(),
            1,
            "Only one is supporting the request from start"
        );
        claim!(
            request.supporters.contains(&account1),
            "The request sender supports the request"
        );
    }

    #[concordium_test]
    /// Supports a transfer request by one of owners
    ///
    /// - Only one of not supported owners can support
    fn test_receive_support_transfer_request() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);
        let target_account = AccountAddress([4u8; 32]);
        let transfer_amount = Amount::from_micro_ccd(50);

        // Setup state -----------------
        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);
        owners.insert(account3);

        let mut state_builder = TestStateBuilder::new();
        let mut state = State {
            owners,
            last_request_id: 0,
            requests: state_builder.new_map(),
        };

        //Create a request that supported by account1
        let mut supporters = BTreeSet::new();
        supporters.insert(account1);

        let new_request = TransferRequest {
            transfer_amount,
            target_account,
            supporters,
        };

        let request_id = 1;

        state.requests.insert(request_id, new_request);
        state.last_request_id = request_id;

        let mut host = TestHost::new(state, state_builder);
        //End  setup state-----------------

        // Account2 supports the created request
        let parameter_bytes = to_bytes(&request_id);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_parameter(&parameter_bytes);
        ctx.set_sender(Address::Account(account2));
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(0));

        // Execution
        let res = contract_receive_support_transfer_request(&ctx, &mut host);
        claim!(
            res.is_ok(),
            "Contract receive failed, but it should not have."
        );

        let request = host.state().requests.get(&request_id).unwrap();
        claim!(
            request.supporters.contains(&account1),
            "The request sender supports the request"
        );
        claim!(
            request.supporters.contains(&account2),
            "The request sender supports the request"
        );
    }

    #[concordium_test]
    /// Revoke a transfer request support from a request
    ///
    /// - The supported owner should not be in the list of supporters after revoking support
    fn test_receive_not_support_transfer_request() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);
        let target_account = AccountAddress([4u8; 32]);
        let transfer_amount = Amount::from_micro_ccd(50);

        // Setup state -----------------
        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);
        owners.insert(account3);

        let mut state_builder = TestStateBuilder::new();
        let mut state = State {
            owners,
            last_request_id: 0,
            requests: state_builder.new_map(),
        };

        //Create a request that supported by account1
        let mut supporters = BTreeSet::new();
        supporters.insert(account1);

        let new_request = TransferRequest {
            transfer_amount,
            target_account,
            supporters,
        };

        let request_id = 1;

        state.requests.insert(request_id, new_request);
        state.last_request_id = request_id;

        let mut host = TestHost::new(state, state_builder);
        //End  setup state-----------------

        // Account2 supports the created request
        let parameter_bytes = to_bytes(&request_id);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_parameter(&parameter_bytes);
        ctx.set_sender(Address::Account(account1));
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(0));

        // Execution
        let res = contract_receive_not_support_transfer_request(&ctx, &mut host);
        claim!(
            res.is_ok(),
            "Contract receive failed, but it should not have."
        );

        let request = host.state().requests.get(&request_id).unwrap();
        claim!(
            !request.supporters.contains(&account1),
            "The request sender should not support the request, but supports"
        );
    }

    #[concordium_test]
    /// Execute transfer for invalid requestId
    ///
    /// - The transaction is rejected with RequestNotFound error
    fn test_receive_execute_for_non_existing_request() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);
        let target_account = AccountAddress([4u8; 32]);
        let transfer_amount = Amount::from_micro_ccd(50);

        // Setup state -----------------
        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);
        owners.insert(account3);

        let mut state_builder = TestStateBuilder::new();
        let mut state = State {
            owners,
            last_request_id: 0,
            requests: state_builder.new_map(),
        };

        //Create a request that supported by account1
        let mut supporters = BTreeSet::new();
        supporters.insert(account1);
        supporters.insert(account2);
        supporters.insert(account3);

        let new_request = TransferRequest {
            transfer_amount,
            target_account,
            supporters,
        };

        let request_id = 1;

        state.requests.insert(request_id, new_request);
        state.last_request_id = request_id;

        let mut host = TestHost::new(state, state_builder);
        //End  setup state-----------------

        let request_id_to_execute = 2;
        let parameter_bytes = to_bytes(&request_id_to_execute);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_parameter(&parameter_bytes);
        ctx.set_sender(Address::Account(account2));
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(0));

        // Execution
        let res = contract_receive_execute_transfer_request(&ctx, &mut host);
        claim!(!res.is_ok(), "Contract should failed");
    }

    #[concordium_test]
    /// Execute a transfer request without getting support from all owners
    ///
    /// - The transaction is rejected with RequestNotSupportedByAllOwners error
    fn test_receive_execute_when_not_all_owners_support() {
        // Setup context
        let account1 = AccountAddress([1u8; 32]);
        let account2 = AccountAddress([2u8; 32]);
        let account3 = AccountAddress([3u8; 32]);
        let target_account = AccountAddress([4u8; 32]);
        let transfer_amount = Amount::from_micro_ccd(50);

        // Setup state -----------------
        let mut owners = BTreeSet::new();
        owners.insert(account1);
        owners.insert(account2);

        let mut state_builder = TestStateBuilder::new();
        let mut state = State {
            owners,
            last_request_id: 0,
            requests: state_builder.new_map(),
        };

        //Create a request that supported by account1
        let mut supporters = BTreeSet::new();
        supporters.insert(account1);
        supporters.insert(account2);
        supporters.insert(account3);

        let new_request = TransferRequest {
            transfer_amount,
            target_account,
            supporters,
        };

        let request_id = 1;

        state.requests.insert(request_id, new_request);
        state.last_request_id = request_id;

        let mut host = TestHost::new(state, state_builder);
        //End  setup state-----------------

        let parameter_bytes = to_bytes(&request_id);

        let mut ctx = TestReceiveContext::empty();
        ctx.set_parameter(&parameter_bytes);
        ctx.set_sender(Address::Account(account2));
        ctx.metadata_mut()
            .set_slot_time(Timestamp::from_timestamp_millis(0));

        // Execution
        let res = contract_receive_execute_transfer_request(&ctx, &mut host);
        claim!(!res.is_ok(), "Contract should failed");
    }
}
