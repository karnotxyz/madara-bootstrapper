pub mod contract_clients;
pub mod helpers;
mod setup_scripts;
#[cfg(test)]
pub mod tests;
pub mod utils;

use clap::Parser;
use dotenv::dotenv;
use inline_colorization::*;
use starknet_accounts::Account;
use starknet_ff::FieldElement;

use crate::contract_clients::config::Config;
use crate::contract_clients::core_contract::CoreContract;
use crate::contract_clients::eth_bridge::StarknetLegacyEthBridge;
use crate::contract_clients::token_bridge::StarknetTokenBridge;
use crate::setup_scripts::account_setup::account_init;
use crate::setup_scripts::argent::ArgentSetup;
use crate::setup_scripts::braavos::BraavosSetup;
use crate::setup_scripts::core_contract::CoreContractStarknetL1;
use crate::setup_scripts::erc20_bridge::Erc20Bridge;
use crate::setup_scripts::eth_bridge::EthBridge;
use crate::setup_scripts::udc::UdcSetup;
use crate::utils::banner::BANNER;
use crate::utils::{save_to_json, JsonValueType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[clap(long, env, default_value = "http://127.0.0.1:8545")]
    eth_rpc: String,
    #[clap(long, env, default_value = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")]
    eth_priv_key: String,
    #[clap(long, env, default_value = "http://127.0.0.1:9944")]
    rollup_seq_url: String,
    #[clap(long, env, default_value = "0xabcd")]
    rollup_priv_key: String,
    #[clap(long, env, default_value_t = 31337)]
    eth_chain_id: u64,
    #[clap(long, env, default_value = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")]
    l1_deployer_address: String,
    #[clap(long, env, default_value = "15")]
    l1_wait_time: String,
    #[clap(long, env, default_value = "0x41fc2a467ef8649580631912517edcab7674173f1dbfa2e9b64fbcd82bc4d79")]
    sn_os_program_hash: String,
    #[clap(long, env, default_value = "StarknetOsConfig1")]
    config_hash_version: String,
    #[clap(long, env, default_value = "MADARA")]
    app_chain_id: String,
    #[clap(long, env, default_value = "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")]
    fee_token_address: String,
    #[clap(long, env, default_value_t = 80)]
    cross_chain_wait_time: u64,
    // Default test address value taken from anvil
    // IMP : Not to be used in prod environment
    #[clap(long, env, default_value = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8")]
    l1_multisig_address: String,
    // Default test address value taken from starknet-devnet
    // IMP : Not to be used in prod environment
    #[clap(long, env, default_value = "0x556455b8ac8bc00e0ad061d7df5458fa3c372304877663fa21d492a8d5e9435")]
    l2_multisig_address: String,
    // Given as 0xabcd by default
    #[clap(long, env, default_value = "0x000000000000000000000000000000000000abcd")]
    verifier_address: String,
    // Given as 0xabcd by default
    #[clap(long, env, default_value = "0x000000000000000000000000000000000000abcd")]
    operator_address: String,
    #[clap(long, default_value = "false")]
    dev: bool,
}

#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok();

    let args = CliArgs::parse();

    bootstrap(&args).await;
}

pub struct DeployBridgeOutput {
    pub starknet_contract: Box<dyn CoreContract>,
    pub starknet_token_bridge: StarknetTokenBridge,
    pub erc20_class_hash: FieldElement,
    pub legacy_eth_bridge_class_hash: FieldElement,
    pub account_address: FieldElement,
    pub eth_proxy_address: FieldElement,
    pub eth_bridge_proxy_address: FieldElement,
    pub legacy_proxy_class_hash: FieldElement,
    pub starkgate_proxy_class_hash: FieldElement,
    pub erc20_legacy_class_hash: FieldElement,
    pub erc20_l2_bridge_address: FieldElement,
    pub l2_erc20_token_address: FieldElement,
    pub eth_bridge: StarknetLegacyEthBridge,
}

pub async fn bootstrap(config: &CliArgs) -> DeployBridgeOutput {
    println!("{color_red}{}{color_reset}", BANNER);
    let clients = Config::init(config).await;
    let core_contract = CoreContractStarknetL1::new(config, &clients);
    let core_contract_client = core_contract.setup().await;
    log::info!("📦 Core address : {:?}", core_contract_client.core_contract_client.address());
    save_to_json(
        "l1_core_contract_address",
        &JsonValueType::EthAddress(core_contract_client.core_contract_client.address()),
    )
    .unwrap();
    log::info!("✅ Core setup init for L1 successful.");
    log::info!("⏳ L2 State and Initialisation Started");
    let account = account_init(&clients, config).await;
    log::info!("🔐 Account with given  private key deployed on L2. [Account Address : {:?}]", account.address());
    log::info!("⏳ Starting ETH bridge deployment");
    let eth_bridge = EthBridge::new(
        account.clone(),
        account.address(),
        config,
        &clients,
        core_contract_client.core_contract_client.as_ref(),
    );
    let eth_bridge_setup_outputs = eth_bridge.setup().await;
    log::info!("✅ ETH bridge deployment complete.");
    log::info!("⏳ Starting ERC20 token bridge deployment");
    let erc20_bridge = Erc20Bridge::new(
        account.clone(),
        account.address(),
        config,
        &clients,
        core_contract_client.core_contract_client.as_ref(),
    );
    let erc20_bridge_setup_outputs = erc20_bridge.setup().await;
    log::info!("✅ ERC20 token bridge deployment complete.");
    log::info!("⏳ Starting UDC (Universal Deployer Contract) deployment");
    let udc = UdcSetup::new(account.clone(), account.address(), config);
    let udc_setup_outputs = udc.setup().await;
    log::info!(
        "*️⃣ UDC setup completed. [UDC Address : {:?}, UDC class hash : {:?}]",
        udc_setup_outputs.udc_address,
        udc_setup_outputs.udc_class_hash
    );
    log::info!("✅ UDC (Universal Deployer Contract) deployment complete.");
    log::info!("⏳ Starting Argent Account declaration");
    let argent = ArgentSetup::new(account.clone());
    let argent_setup_outputs = argent.setup().await;
    log::info!("*️⃣ Argent setup completed. [Argent account class hash : {:?}]", argent_setup_outputs.argent_class_hash);
    log::info!("✅ Argent Account declaration complete.");
    log::info!("⏳ Starting Braavos Account declaration");
    let braavos = BraavosSetup::new(account.clone(), config);
    let braavos_setup_outputs = braavos.setup().await;
    log::info!(
        "*️⃣ Braavos setup completed. [Braavos account class hash : {:?}]",
        braavos_setup_outputs.braavos_class_hash
    );
    log::info!("✅ Braavos Account declaration complete.");

    DeployBridgeOutput {
        starknet_contract: core_contract_client.core_contract_client,
        starknet_token_bridge: erc20_bridge_setup_outputs.starknet_token_bridge,
        erc20_class_hash: erc20_bridge_setup_outputs.erc20_cairo_one_class_hash,
        legacy_eth_bridge_class_hash: eth_bridge_setup_outputs.legacy_eth_bridge_class_hash,
        account_address: account.address(),
        eth_proxy_address: eth_bridge_setup_outputs.eth_proxy_address,
        eth_bridge_proxy_address: eth_bridge_setup_outputs.eth_bridge_proxy_address,
        legacy_proxy_class_hash: eth_bridge_setup_outputs.legacy_proxy_class_hash,
        starkgate_proxy_class_hash: eth_bridge_setup_outputs.starkgate_proxy_class_hash,
        erc20_legacy_class_hash: eth_bridge_setup_outputs.erc20_legacy_class_hash,
        erc20_l2_bridge_address: erc20_bridge_setup_outputs.erc20_l2_bridge_address,
        l2_erc20_token_address: erc20_bridge_setup_outputs.l2_erc20_token_address,
        eth_bridge: eth_bridge_setup_outputs.eth_bridge,
    }
}
