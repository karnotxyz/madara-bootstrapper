use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use ethers::abi::AbiEncode;
use ethers::addressbook::Address;
use ethers::prelude::{Bytes, I256, U256};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_ff::FieldElement;
use starknet_proxy_client::proxy_support::{CoreContractInitData, CoreContractState};
use zaun_utils::LocalWalletSignerMiddleware;

use crate::contract_clients::config::Config;
use crate::contract_clients::starknet_sovereign::StarknetSovereignContract;
use crate::contract_clients::starknet_validity::StarknetValidityContract;
use crate::utils::convert_felt_to_u256;

pub enum CoreContractStarknetL1Enum {
    Sovereign(StarknetSovereignContract),
    Validity(StarknetValidityContract),
}

#[async_trait]
impl CoreContract for CoreContractStarknetL1Enum {
    fn address(&self) -> Address {
        match self {
            Self::Sovereign(contract) => contract.address(),
            Self::Validity(contract) => contract.address(),
        }
    }

    fn implementation_address(&self) -> Address {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => contract.implementation_address(),
            CoreContractStarknetL1Enum::Validity(contract) => contract.implementation_address(),
        }
    }

    fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => contract.client(),
            CoreContractStarknetL1Enum::Validity(contract) => contract.client(),
        }
    }

    async fn initialize_with(&self, init_data: CoreContractInitData) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => contract.initialize_with(init_data).await,
            CoreContractStarknetL1Enum::Validity(contract) => contract.initialize_with(init_data).await,
        }
    }

    async fn add_implementation_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract
                    .add_implementation_core_contract(
                        block_number,
                        state_root,
                        program_hash,
                        config_hash,
                        implementation_address,
                        verifier_address,
                        finalized,
                    )
                    .await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract
                    .add_implementation_core_contract(
                        block_number,
                        state_root,
                        program_hash,
                        config_hash,
                        implementation_address,
                        verifier_address,
                        finalized,
                    )
                    .await
            }
        }
    }

    async fn upgrade_to_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    ) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract
                    .upgrade_to_core_contract(
                        block_number,
                        state_root,
                        program_hash,
                        config_hash,
                        implementation_address,
                        verifier_address,
                        finalized,
                    )
                    .await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract
                    .upgrade_to_core_contract(
                        block_number,
                        state_root,
                        program_hash,
                        config_hash,
                        implementation_address,
                        verifier_address,
                        finalized,
                    )
                    .await
            }
        }
    }

    async fn register_operator_core_contract(&self, operator_address: Address) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract.register_operator_core_contract(operator_address).await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract.register_operator_core_contract(operator_address).await
            }
        }
    }

    async fn nominate_governor_core_contract(&self, l1_governor_address: Address) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract.nominate_governor_core_contract(l1_governor_address).await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract.nominate_governor_core_contract(l1_governor_address).await
            }
        }
    }

    async fn nominate_governor_core_contract_proxy(&self, l1_governor_address: Address) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract.nominate_governor_core_contract_proxy(l1_governor_address).await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract.nominate_governor_core_contract_proxy(l1_governor_address).await
            }
        }
    }

    async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => contract.initialize(program_hash, config_hash).await,
            CoreContractStarknetL1Enum::Validity(contract) => contract.initialize(program_hash, config_hash).await,
        }
    }

    async fn initialize_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        verifer_address: Address,
    ) {
        match self {
            CoreContractStarknetL1Enum::Sovereign(contract) => {
                contract
                    .initialize_core_contract(block_number, state_root, program_hash, config_hash, verifer_address)
                    .await
            }
            CoreContractStarknetL1Enum::Validity(contract) => {
                contract
                    .initialize_core_contract(block_number, state_root, program_hash, config_hash, verifer_address)
                    .await
            }
        }
    }
}

#[async_trait]
pub trait CoreContract {
    fn address(&self) -> Address;

    fn implementation_address(&self) -> Address;

    fn client(&self) -> Arc<LocalWalletSignerMiddleware>;

    async fn initialize_with(&self, init_data: CoreContractInitData);

    #[allow(clippy::too_many_arguments)]
    async fn add_implementation_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    );

    #[allow(clippy::too_many_arguments)]
    async fn upgrade_to_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        implementation_address: Address,
        verifier_address: Address,
        finalized: bool,
    );

    async fn register_operator_core_contract(&self, operator_address: Address);

    async fn nominate_governor_core_contract(&self, l1_governor_address: Address);

    async fn nominate_governor_core_contract_proxy(&self, l1_governor_address: Address);

    async fn initialize(&self, program_hash: StarkFelt, config_hash: StarkFelt);

    async fn initialize_core_contract(
        &self,
        block_number: StarkFelt,
        state_root: StarkFelt,
        program_hash: FieldElement,
        config_hash: StarkHash,
        verifer_address: Address,
    );
}

pub trait CoreContractDeploy<T> {
    fn deploy(config: &Config) -> impl Future<Output = T> + Send;
}

pub fn get_init_data_core_contract(
    block_number: StarkFelt,
    state_root: StarkFelt,
    program_hash: StarkFelt,
    config_hash: StarkHash,
    verifier_address: Address,
) -> CoreContractInitData {
    CoreContractInitData {
        program_hash: convert_felt_to_u256(program_hash), // zero program hash would be deemed invalid
        verifier_address,
        config_hash: convert_felt_to_u256(config_hash),
        // TODO :
        // Figure out the exact params for production env
        initial_state: CoreContractState {
            block_number: I256::from_raw(convert_felt_to_u256(block_number)),
            state_root: convert_felt_to_u256(state_root),
            // TODO :
            // Remove hardcoded values.
            block_hash: U256::zero(),
        },
    }
}

pub fn get_calldata_bytes(calldata: CoreContractInitData) -> Bytes {
    let mut bytes_final = Address::zero().encode();
    let bytes: Vec<u8> = <CoreContractInitData as Into<Vec<u8>>>::into(calldata.clone());
    for x in bytes {
        bytes_final.push(x);
    }

    Bytes::from(bytes_final)
}
