use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
};

use crate::msg::{ExecuteMsg, QueryMsg, UserResponse, UsersResponse, FriendsResponse, PaymentResponse, PaymentsResponse};
use crate::error::ContractError;

/// SocialPaymentContract is a wrapper around Addr that provides helpers for your contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SocialPaymentContract(pub Addr);

impl SocialPaymentContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    /// Query a user by username
    pub fn get_user_by_username<Q, CQ>(&self, querier: &Q, username: String) -> StdResult<UserResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetUserByUsername { username };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: UserResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }

    /// Query a user by wallet address
    pub fn get_user_by_wallet<Q, CQ>(&self, querier: &Q, wallet_address: String) -> StdResult<UserResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetUserByWallet { wallet_address };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: UserResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }

    /// Search users by query string
    pub fn search_users<Q, CQ>(&self, querier: &Q, query: String) -> StdResult<UsersResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::SearchUsers { query };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: UsersResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }

    /// Query user's friends
    pub fn get_user_friends<Q, CQ>(&self, querier: &Q, username: String) -> StdResult<FriendsResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetUserFriends { username };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: FriendsResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }

    /// Query a payment by ID
    pub fn get_payment_by_id<Q, CQ>(&self, querier: &Q, payment_id: u64) -> StdResult<PaymentResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetPaymentById { payment_id };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: PaymentResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }

    /// Query payment history for a user
    pub fn get_payment_history<Q, CQ>(&self, querier: &Q, username: String) -> StdResult<PaymentsResponse>
    where
        Q: Querier,
        CQ: CustomQuery,
    {
        let msg = QueryMsg::GetPaymentHistory { username };
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&msg)?,
        }
        .into();
        let res: PaymentsResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
        Ok(res)
    }
}

/// zkTLS verification interface - stubbed for now
pub fn verify_zktls(proof_blob: &str, endpoint: &str) -> Result<bool, ContractError> {
    // TODO: Replace with actual zkTLS verification logic
    // For now, this is a stub that can be easily swapped out
    
    // Basic validation checks
    if proof_blob.is_empty() || endpoint.is_empty() {
        return Err(ContractError::InvalidProof {});
    }
    
    // Stub implementation - in production, this would:
    // 1. Parse the zkTLS proof
    // 2. Verify the proof cryptographically
    // 3. Check that the proof corresponds to the expected endpoint
    // 4. Validate the proof's timestamp and other metadata
    
    // For testing/development, we'll consider proofs valid if they contain "valid"
    let is_valid = proof_blob.contains("valid") || proof_blob.len() > 10;
    
    Ok(is_valid)
}

/// Hash a piece of data for on-chain storage
pub fn hash_data(data: &str) -> String {
    // Simple hash for now - in production use proper cryptographic hash
    format!("hash_{}", data.len())
}
