use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use proof_of_work_contract::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, UserResponse, UsersResponse, PaymentResponse, PaymentsResponse,
    UsernameResponse, WalletResponse, HasUsernameResponse, UsernameAvailableResponse
};
use proof_of_work_contract::state::{User, Payment, PaymentStatus, ProofType, State};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(UserResponse), &out_dir);
    export_schema(&schema_for!(UsersResponse), &out_dir);
    export_schema(&schema_for!(PaymentResponse), &out_dir);
    export_schema(&schema_for!(PaymentsResponse), &out_dir);
    export_schema(&schema_for!(UsernameResponse), &out_dir);
    export_schema(&schema_for!(WalletResponse), &out_dir);
    export_schema(&schema_for!(HasUsernameResponse), &out_dir);
    export_schema(&schema_for!(UsernameAvailableResponse), &out_dir);
    export_schema(&schema_for!(User), &out_dir);
    export_schema(&schema_for!(Payment), &out_dir);
    export_schema(&schema_for!(PaymentStatus), &out_dir);
    export_schema(&schema_for!(ProofType), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
}
