use crate::bridge::contract_clients::config::{build_single_owner_account, field_element_to_u256};
use crate::bridge::helpers::account_actions::{
    get_contract_address_from_deploy_tx, AccountActions,
};
use crate::utils::constants::LEGACY_BRIDGE_PATH;
use crate::utils::utils::{invoke_contract, wait_for_transaction};
use async_trait::async_trait;
use ethers::addressbook::Address;
use ethers::providers::Middleware;
use ethers::types::{Bytes, U256};
use starknet_accounts::Account;
use starknet_contract::ContractFactory;
use starknet_eth_bridge_client::clients::eth_bridge::StarknetEthBridgeContractClient;
use starknet_eth_bridge_client::deploy_starknet_eth_bridge_behind_unsafe_proxy;
use starknet_eth_bridge_client::interfaces::eth_bridge::StarknetEthBridgeTrait;
use starknet_ff::FieldElement;
use starknet_providers::jsonrpc::HttpTransport;
use starknet_providers::JsonRpcClient;
use starknet_proxy_client::proxy_support::ProxySupportTrait;
use std::sync::Arc;
use zaun_utils::{LocalWalletSignerMiddleware, StarknetContractClient};

#[async_trait]
pub trait BridgeDeployable {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self;
}

pub struct StarknetLegacyEthBridge {
    eth_bridge: StarknetEthBridgeContractClient,
}

#[async_trait]
impl BridgeDeployable for StarknetLegacyEthBridge {
    async fn deploy(client: Arc<LocalWalletSignerMiddleware>) -> Self {
        let eth_bridge = deploy_starknet_eth_bridge_behind_unsafe_proxy(client.clone())
            .await
            .expect("Failed to deploy starknet contract");

        Self { eth_bridge }
    }
}

impl StarknetLegacyEthBridge {
    pub fn address(&self) -> Address {
        self.eth_bridge.address()
    }

    pub fn client(&self) -> Arc<LocalWalletSignerMiddleware> {
        self.eth_bridge.client()
    }

    pub async fn deploy_l2_contracts(
        rpc_provider_l2: &JsonRpcClient<HttpTransport>,
        private_key: &str,
        l2_deployer_address: &str,
    ) -> FieldElement {
        let account =
            build_single_owner_account(&rpc_provider_l2, private_key, l2_deployer_address, false);

        let contract_artifact = account.declare_contract_params_legacy(LEGACY_BRIDGE_PATH);
        let class_hash = contract_artifact.class_hash().unwrap();

        let declare_txn = account
            .declare_legacy(Arc::new(contract_artifact))
            .send()
            .await
            .expect("Unable to declare legacy eth bridge on l2");
        wait_for_transaction(rpc_provider_l2, declare_txn.transaction_hash)
            .await
            .unwrap();

        let contract_factory = ContractFactory::new(class_hash, account.clone());
        let deploy_tx = &contract_factory
            .deploy(vec![], FieldElement::ZERO, true)
            .send()
            .await
            .expect("Unable to deploy legacy eth bridge on l2");

        wait_for_transaction(rpc_provider_l2, deploy_tx.transaction_hash)
            .await
            .unwrap();

        let address = get_contract_address_from_deploy_tx(&rpc_provider_l2, deploy_tx)
            .await
            .expect("Error getting contract address from transaction hash");
        address
    }

    /// Initialize Starknet Legacy Eth Bridge
    pub async fn initialize(&self, messaging_contract: Address) {
        let empty_bytes = [0u8; 32];

        let messaging_bytes = messaging_contract.as_bytes();

        let mut padded_messaging_bytes = Vec::with_capacity(32);
        padded_messaging_bytes.extend(vec![0u8; 32 - messaging_bytes.len()]);
        padded_messaging_bytes.extend_from_slice(messaging_bytes);

        let mut calldata = Vec::new();
        calldata.extend(empty_bytes);
        calldata.extend(empty_bytes);
        calldata.extend(padded_messaging_bytes);

        self.eth_bridge
            .initialize(Bytes::from(calldata))
            .await
            .expect("Failed to initialize eth bridge");
    }

    /// Sets up the Eth bridge with the specified data
    pub async fn setup_l1_bridge(
        &self,
        max_total_balance: &str,
        max_deposit: &str,
        l2_bridge: FieldElement,
    ) {
        self.eth_bridge
            .set_max_total_balance(U256::from_dec_str(max_total_balance).unwrap())
            .await
            .unwrap();
        self.eth_bridge
            .set_max_deposit(U256::from_dec_str(max_deposit).unwrap())
            .await
            .unwrap();
        self.eth_bridge
            .set_l2_token_bridge(field_element_to_u256(l2_bridge))
            .await
            .unwrap();
    }

    pub async fn setup_l2_bridge(
        &self,
        rpc_provider: &JsonRpcClient<HttpTransport>,
        l2_bridge_address: FieldElement,
        erc20_address: FieldElement,
        priv_key: &str,
        l2_deployer_address: &str,
    ) {
        let tx = invoke_contract(
            rpc_provider,
            l2_bridge_address,
            "initialize",
            vec![
                FieldElement::from_dec_str("1").unwrap(),
                FieldElement::from_hex_be(l2_deployer_address).unwrap(),
            ],
            priv_key,
            l2_deployer_address,
        )
        .await;

        log::trace!("setup_l2_bridge : l2 bridge initialized //");
        wait_for_transaction(rpc_provider, tx.transaction_hash)
            .await
            .unwrap();
        // sleep(Duration::from_secs(7)).await;

        let tx = invoke_contract(
            rpc_provider,
            l2_bridge_address,
            "set_l2_token",
            vec![erc20_address],
            priv_key,
            l2_deployer_address,
        )
        .await;

        log::trace!("setup_l2_bridge : l2 token set //");
        wait_for_transaction(rpc_provider, tx.transaction_hash)
            .await
            .unwrap();
        // sleep(Duration::from_secs(7)).await;

        let tx = invoke_contract(
            rpc_provider,
            l2_bridge_address,
            "set_l1_bridge",
            vec![FieldElement::from_byte_slice_be(self.eth_bridge.address().as_bytes()).unwrap()],
            priv_key,
            l2_deployer_address,
        )
        .await;

        log::trace!("setup_l2_bridge : l1 bridge set //");
        wait_for_transaction(rpc_provider, tx.transaction_hash)
            .await
            .unwrap();
    }

    pub async fn set_max_total_balance(&self, amount: U256) {
        self.eth_bridge
            .set_max_total_balance(amount)
            .await
            .expect("Failed to set max total balance value in Eth bridge");
    }

    pub async fn set_max_deposit(&self, amount: U256) {
        self.eth_bridge
            .set_max_deposit(amount)
            .await
            .expect("Failed to set max deposit value in eth bridge");
    }

    pub async fn set_l2_token_bridge(&self, l2_bridge: U256) {
        self.eth_bridge
            .set_l2_token_bridge(l2_bridge)
            .await
            .expect("Failed to set l2 bridge in eth bridge");
    }

    pub async fn deposit(&self, amount: U256, l2_address: U256, fee: U256) {
        self.eth_bridge
            .deposit(amount, l2_address, fee)
            .await
            .expect("Failed to deposit in eth bridge");
    }

    pub async fn withdraw(&self, amount: U256, l1_recipient: Address) {
        self.eth_bridge
            .withdraw(amount, l1_recipient)
            .await
            .expect("Failed to withdraw from eth bridge");
    }

    pub async fn eth_balance(&self, l1_recipient: Address) -> U256 {
        let provider = self.eth_bridge.client().provider().clone();

        provider.get_balance(l1_recipient, None).await.unwrap()
    }
}