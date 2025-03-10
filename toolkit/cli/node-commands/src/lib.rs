use authority_selection_inherents::authority_selection_inputs::AuthoritySelectionDataSource;
use authority_selection_inherents::authority_selection_inputs::AuthoritySelectionInputs;
use authority_selection_inherents::filter_invalid_candidates::CandidateValidationApi;
use clap::Parser;
use cli_commands::registration_signatures::RegistrationSignaturesCmd;
use frame_support::sp_runtime::traits::NumberFor;
use parity_scale_codec::{Decode, Encode};
use partner_chains_smart_contracts_commands::SmartContractsCmd;
use sc_cli::{CliConfiguration, SharedParams, SubstrateCli};
use sc_service::TaskManager;
use sidechain_domain::{MainchainPublicKey, McEpochNumber, ScEpochNumber};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_session_validator_management::SessionValidatorManagementApi;
use sp_session_validator_management_query::commands::*;
use sp_session_validator_management_query::SessionValidatorManagementQuery;
use sp_sidechain::GetGenesisUtxo;
use sp_sidechain::GetSidechainStatus;
use std::future::Future;
use std::sync::Arc;

#[derive(Debug, Clone, Parser)]
pub struct AriadneParametersCmd {
	#[arg(long)]
	pub mc_epoch_number: McEpochNumber,
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl CliConfiguration for AriadneParametersCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}

#[derive(Debug, Clone, Parser)]
pub struct SidechainParamsCmd {
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl CliConfiguration for SidechainParamsCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}

#[derive(Debug, Clone, Parser)]
pub struct RegistrationStatusCmd {
	#[arg(long)]
	pub mainchain_pub_key: MainchainPublicKey,
	#[arg(long)]
	pub mc_epoch_number: McEpochNumber,
	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl CliConfiguration for RegistrationStatusCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}

#[derive(Clone, Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum PartnerChainsSubcommand {
	/// Returns sidechain parameters
	SidechainParams(SidechainParamsCmd),

	/// Returns registration status for a given mainchain public key and epoch number.
	/// If registration has been included in Cardano block in epoch N, then it should be returned by this command if epoch greater than N+1 is provided.
	/// If this command won't show your registration after a few minutes after it has been included in a cardano block, you can start debugging for unsuccessful registration.
	#[clap(
		after_help = "Example: partner-chains-node -- registration-status --mainchain-pub-key 0x702b81ab2e86cf73a87062af1eb0da666d451976d9d91c63a119ed94e6a33dc0 --mc-epoch-number 586"
	)]
	RegistrationStatus(RegistrationStatusCmd),

	/// Returns ariadne parameters effective at given mainchain epoch number.
	/// Parameters are effective two epochs after the block their change is included in.
	AriadneParameters(AriadneParametersCmd),

	/// Generates registration signatures for partner chains committee candidates
	RegistrationSignatures(RegistrationSignaturesCmd),

	/// Commands for interacting with Partner Chain smart contracts on Cardano
	#[command(subcommand)]
	SmartContracts(SmartContractsCmd),
}

pub fn run<Cli, Block, CrossChainPublic, SessionKeys, Client>(
	cli: &Cli,
	get_deps: impl FnOnce(
		sc_service::Configuration,
	) -> Result<
		(Arc<Client>, TaskManager, Arc<dyn AuthoritySelectionDataSource + Send + Sync>),
		sc_service::error::Error,
	>,
	cmd: PartnerChainsSubcommand,
) -> sc_cli::Result<()>
where
	Cli: SubstrateCli,
	Client: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	Client::Api: GetGenesisUtxo<Block>
		+ GetSidechainStatus<Block>
		+ SessionValidatorManagementApi<
			Block,
			SessionKeys,
			CrossChainPublic,
			AuthoritySelectionInputs,
			ScEpochNumber,
		> + CandidateValidationApi<Block>,
	Block: BlockT,
	NumberFor<Block>: From<u32> + Into<u32>,
	SessionKeys: Decode + Send + Sync + 'static,
	CrossChainPublic: Decode + Encode + AsRef<[u8]> + Send + Sync + 'static,
{
	match cmd {
		PartnerChainsSubcommand::SidechainParams(cmd) => {
			let runner = cli.create_runner(&cmd)?;
			runner.async_run(|config| {
				let (client, task_manager, _) = get_deps(config)?;
				Ok((print_result(sp_sidechain::query::get_genesis_utxo(client)), task_manager))
			})
		},
		PartnerChainsSubcommand::RegistrationStatus(cmd) => {
			let runner = cli.create_runner(&cmd)?;
			runner.async_run(move |config| {
				let (client, task_manager, ds) = get_deps(config)?;
				let query = SessionValidatorManagementQuery::new(client.clone(), ds.clone());
				Ok((
					print_result(cli_get_registration_status(
						query,
						cmd.mc_epoch_number,
						cmd.mainchain_pub_key.clone(),
					)),
					task_manager,
				))
			})
		},
		PartnerChainsSubcommand::AriadneParameters(cmd) => {
			let runner = cli.create_runner(&cmd)?;
			runner.async_run(move |config| {
				let (client, task_manager, ds) = get_deps(config)?;
				let query = SessionValidatorManagementQuery::new(client.clone(), ds.clone());
				Ok((
					print_result(cli_get_ariadne_parameters(query, cmd.mc_epoch_number)),
					task_manager,
				))
			})
		},
		PartnerChainsSubcommand::RegistrationSignatures(cmd) => Ok(println!("{}", cmd.execute())),
		PartnerChainsSubcommand::SmartContracts(cmd) => Ok(cmd.execute_blocking()?),
	}
}

async fn print_result<FIn>(command_future: FIn) -> Result<(), sc_cli::Error>
where
	FIn: Future<Output = Result<String, String>>,
{
	let result = command_future.await.unwrap_or_else(|e| e);
	println!("{}", result);
	Ok(())
}

#[cfg(test)]
mod tests {

	async fn some_err() -> Result<String, String> {
		Err("some err".to_string())
	}

	#[tokio::test]
	async fn print_async_doesnt_fail_if_result_is_error() {
		let result = super::print_result(some_err()).await;
		assert!(result.is_ok());
	}
}
