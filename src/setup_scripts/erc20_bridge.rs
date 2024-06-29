use std::str::FromStr;
use std::time::Duration;

use ethers::abi::Address;
use ethers::prelude::{H160, U256};
use starknet_core::types::{BlockId, BlockTag, FunctionCall};
use starknet_core::utils::get_selector_from_name;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::{JsonRpcClient, Provider};
use tokio::time::sleep;

use crate::contract_clients::config::Config;
use crate::contract_clients::core_contract::CoreContract;
use crate::contract_clients::eth_bridge::BridgeDeployable;
use crate::contract_clients::token_bridge::StarknetTokenBridge;
use crate::contract_clients::utils::{build_single_owner_account, declare_contract, DeclarationInput, RpcAccount};
use crate::utils::constants::{ERC20_CASM_PATH, ERC20_SIERRA_PATH};
use crate::utils::{convert_to_hex, save_to_json, JsonValueType};
use crate::CliArgs;

pub struct Erc20Bridge<'a> {
    account: RpcAccount<'a>,
    account_address: FieldElement,
    arg_config: &'a CliArgs,
    clients: &'a Config,
    core_contract: &'a dyn CoreContract,
}

pub struct Erc20BridgeSetupOutput {
    pub erc20_cairo_one_class_hash: FieldElement,
    pub starknet_token_bridge: StarknetTokenBridge,
    pub erc20_l2_bridge_address: FieldElement,
    pub l2_erc20_token_address: FieldElement,
}

impl<'a> Erc20Bridge<'a> {
    pub fn new(
        account: RpcAccount<'a>,
        account_address: FieldElement,
        arg_config: &'a CliArgs,
        clients: &'a Config,
        core_contract: &'a dyn CoreContract,
    ) -> Self {
        Self { account, account_address, arg_config, clients, core_contract }
    }

    pub async fn setup(&self) -> Erc20BridgeSetupOutput {
        let erc20_cairo_one_class_hash = declare_contract(DeclarationInput::DeclarationInputs(
            String::from(ERC20_SIERRA_PATH),
            String::from(ERC20_CASM_PATH),
            self.account.clone(),
        ))
        .await;
        log::debug!("🌗 ERC20 Class Hash declared : {:?}", erc20_cairo_one_class_hash);
        save_to_json("erc20_cairo_one_class_hash", &JsonValueType::StringType(erc20_cairo_one_class_hash.to_string()))
            .unwrap();
        sleep(Duration::from_secs(10)).await;

        let token_bridge = StarknetTokenBridge::deploy(self.core_contract.client().clone()).await;

        log::info!(
            "❇️ ERC20 Token Bridge L1 deployment completed [ERC20 Token Bridge Address (L1) : {:?}]",
            token_bridge.bridge_address()
        );
        save_to_json("ERC20_l1_bridge_address", &JsonValueType::EthAddress(token_bridge.bridge_address())).unwrap();
        save_to_json("ERC20_l1_registry_address", &JsonValueType::EthAddress(token_bridge.registry_address())).unwrap();
        save_to_json("ERC20_l1_manager_address", &JsonValueType::EthAddress(token_bridge.manager_address())).unwrap();

        let l2_bridge_address = StarknetTokenBridge::deploy_l2_contracts(
            self.clients.provider_l2(),
            &self.arg_config.rollup_priv_key,
            &convert_to_hex(&self.account_address.to_string()),
        )
        .await;

        log::info!(
            "❇️ ERC20 Token Bridge L2 deployment completed [ERC20 Token Bridge Address (L2) : {:?}]",
            l2_bridge_address
        );
        save_to_json("ERC20_l2_bridge_address", &JsonValueType::StringType(l2_bridge_address.to_string())).unwrap();

        let provider_l2 = self.clients.provider_l2();
        let account = build_single_owner_account(
            provider_l2,
            &self.arg_config.rollup_priv_key,
            &convert_to_hex(&self.account_address.to_string()),
            false,
        )
        .await;

        token_bridge.add_implementation_token_bridge(self.core_contract.address()).await;
        token_bridge.upgrade_to_token_bridge(self.core_contract.address()).await;
        token_bridge
            .setup_l2_bridge(
                self.clients.provider_l2(),
                l2_bridge_address,
                &convert_to_hex(&self.account_address.to_string()),
                &account,
                erc20_cairo_one_class_hash,
            )
            .await;
        token_bridge
            .setup_l1_bridge(
                H160::from_str(&self.arg_config.l1_deployer_address).unwrap(),
                l2_bridge_address,
                U256::from_dec_str("100000000000000").unwrap(),
                Address::from_str(&self.arg_config.l1_multisig_address.to_string()).unwrap(),
            )
            .await;
        log::info!("❇️ Temp test token deployed on L1.");
        log::info!(
            "❇️ Waiting for temp test token to be deployed on L2 [⏳....] Approx. time : {:?} secs.",
            self.arg_config.cross_chain_wait_time + 10_u64
        );
        sleep(Duration::from_secs(self.arg_config.l1_wait_time.parse().unwrap())).await;
        // We need to wait a little bit more for message to be consumed and executed
        sleep(Duration::from_secs(self.arg_config.cross_chain_wait_time)).await;

        let l2_erc20_token_address =
            get_l2_token_address(self.clients.provider_l2(), &l2_bridge_address, &token_bridge.address()).await;
        log::info!(
            "❇️ L2 ERC20 Token Address deployed for testing [ ERC20 Test Token Address : {:?}]",
            l2_erc20_token_address
        );
        save_to_json(
            "ERC20_l2_token_address_temp_test",
            &JsonValueType::StringType(l2_erc20_token_address.to_string()),
        )
        .unwrap();

        Erc20BridgeSetupOutput {
            erc20_cairo_one_class_hash,
            starknet_token_bridge: token_bridge,
            erc20_l2_bridge_address: l2_bridge_address,
            l2_erc20_token_address,
        }
    }
}

async fn get_l2_token_address(
    rpc_provider_l2: &JsonRpcClient<HttpTransport>,
    l2_bridge_address: &FieldElement,
    l1_erc_20_address: &H160,
) -> FieldElement {
    rpc_provider_l2
        .call(
            FunctionCall {
                contract_address: *l2_bridge_address,
                entry_point_selector: get_selector_from_name("get_l2_token").unwrap(),
                calldata: vec![FieldElement::from_byte_slice_be(l1_erc_20_address.as_bytes()).unwrap()],
            },
            BlockId::Tag(BlockTag::Pending),
        )
        .await
        .unwrap()[0]
}
