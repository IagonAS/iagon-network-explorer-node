use crate::config::config_fields::{
	CHAIN_ID, GENESIS_COMMITTEE_UTXO, GOVERNANCE_AUTHORITY, THRESHOLD_DENOMINATOR,
	THRESHOLD_NUMERATOR,
};
use crate::config::{config_fields, ConfigFieldDefinition, SidechainParams};
use crate::io::IOContext;
use anyhow::{anyhow, Context};
use serde::de::DeserializeOwned;
use sidechain_domain::{MainchainAddressHash, UtxoId};

pub fn prepare_chain_params<C: IOContext>(context: &C) -> anyhow::Result<SidechainParams> {
	context.eprint(INTRO);
	let governance_authority = match GOVERNANCE_AUTHORITY.load_from_file_and_print(context) {
		Some(ga) => {
			if !context.prompt_yes_no(&is_gov_auth_valid_prompt(), true) {
				establish_governance_authority(context)?
			} else {
				ga
			}
		},
		None => establish_governance_authority(context)?,
	};
	context.eprint(CHAIN_ID_PROMPT);
	let chain_id = CHAIN_ID.prompt_with_default_from_file_parse_and_save(context)?;
	let (genesis_committee_utxo, threshold_numerator, threshold_denominator) =
		silently_fill_legacy_chain_params(context)?;
	Ok(SidechainParams {
		chain_id,
		genesis_committee_utxo,
		threshold_numerator,
		threshold_denominator,
		governance_authority,
	})
}

fn silently_fill_legacy_chain_params(
	context: &impl IOContext,
) -> anyhow::Result<(UtxoId, u64, u64)> {
	let genesis_committee_utxo = GENESIS_COMMITTEE_UTXO
		.default
		.ok_or(anyhow!("Genesis committee utxo should have a default value"))?
		.parse()
		.map_err(anyhow::Error::msg)
		.context("Genesis committee utxo should have a valid default value")?;
	let threshold_numerator = THRESHOLD_NUMERATOR
		.default
		.ok_or(anyhow!("Threshold numerator should have a default value"))?
		.parse()
		.map_err(anyhow::Error::msg)
		.context("Threshold numerator default should be a number")?;
	let threshold_denominator = THRESHOLD_DENOMINATOR
		.default
		.ok_or(anyhow!("Threshold denominator should have a default value"))?
		.parse()
		.map_err(anyhow::Error::msg)
		.context("Threshold denominator default should be a number")?;

	save_if_missing(GENESIS_COMMITTEE_UTXO, genesis_committee_utxo, context);
	save_if_missing(THRESHOLD_NUMERATOR, threshold_numerator, context);
	save_if_missing(THRESHOLD_DENOMINATOR, threshold_denominator, context);
	Ok((genesis_committee_utxo, threshold_numerator, threshold_denominator))
}

fn save_if_missing<T, C: IOContext>(field: ConfigFieldDefinition<'_, T>, new_value: T, context: &C)
where
	T: DeserializeOwned + std::fmt::Display + serde::Serialize,
{
	if field.load_from_file_and_print(context).is_none() {
		field.save_to_file(&new_value, context);
	}
}

fn establish_governance_authority(
	context: &impl IOContext,
) -> anyhow::Result<MainchainAddressHash> {
	let cardano_cli =
		config_fields::CARDANO_CLI.prompt_with_default_from_file_parse_and_save(context)?;
	let cardano_payment_verification_key_file =
		config_fields::CARDANO_PAYMENT_VERIFICATION_KEY_FILE
			.prompt_with_default_from_file_and_save(context);
	if !context.file_exists(&cardano_cli) {
		return Err(anyhow!("Cardano CLI executable file ({cardano_cli}) is missing"));
	}

	let governance_authority = context.run_command(&format!("{cardano_cli} address key-hash --payment-verification-key-file {cardano_payment_verification_key_file}"))?;
	let ga = MainchainAddressHash::decode_hex(governance_authority.trim())
		.map_err(|err| anyhow!(err))
		.context("Failed to parse governance authority")?;
	GOVERNANCE_AUTHORITY.save_to_file(&ga, context);
	context.eprint(&format!("Governance authority has been set to {}", ga));
	Ok(ga)
}

const INTRO: &str = "Now, let's set up the chain parameters: chain id and governance authority. The Pair `(governance authority, chain id)` identifies a partner chain, thus it has to be unique.";
const CHAIN_ID_PROMPT: &str = "Now, let's set up chain id. It has to fall in range 0-65535";

fn is_gov_auth_valid_prompt() -> String {
	format!("Is the {} displayed above correct?", GOVERNANCE_AUTHORITY.name)
}

#[cfg(test)]
mod tests {
	use crate::config::config_fields::{
		CARDANO_CLI, CARDANO_PAYMENT_VERIFICATION_KEY_FILE, CHAIN_ID, GENESIS_COMMITTEE_UTXO,
		GOVERNANCE_AUTHORITY, THRESHOLD_DENOMINATOR, THRESHOLD_NUMERATOR,
	};
	use crate::prepare_configuration::prepare_chain_params::tests::scenarios::silently_fill_legacy_chain_params;
	use crate::prepare_configuration::prepare_chain_params::{
		is_gov_auth_valid_prompt, prepare_chain_params, CHAIN_ID_PROMPT,
	};
	use crate::prepare_configuration::tests::{
		prompt_and_save_to_existing_file, prompt_with_default_and_save_to_existing_file,
		save_to_existing_file, save_to_new_file, CHAIN_CONFIG_PATH,
	};
	use crate::tests::{MockIO, MockIOContext};
	use serde_json::Value;
	use sidechain_domain::{MainchainAddressHash, UtxoId};
	use std::str::FromStr;

	const GOV_AUTH: &str = "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9";

	mod scenarios {
		use super::*;
		use crate::prepare_configuration::prepare_chain_params::INTRO;
		use crate::tests::MockIO;

		pub fn show_intro() -> MockIO {
			MockIO::Group(vec![MockIO::eprint(INTRO)])
		}

		pub fn silently_fill_legacy_chain_params(final_chain_config: Value) -> MockIO {
			MockIO::Group(vec![
				MockIO::file_read(GENESIS_COMMITTEE_UTXO.config_file),
				save_to_existing_file(
					GENESIS_COMMITTEE_UTXO,
					GENESIS_COMMITTEE_UTXO.default.unwrap(),
				),
				MockIO::file_read(THRESHOLD_NUMERATOR.config_file),
				save_to_existing_file(THRESHOLD_NUMERATOR, THRESHOLD_NUMERATOR.default.unwrap()),
				MockIO::file_read(THRESHOLD_DENOMINATOR.config_file),
				MockIO::file_read(THRESHOLD_DENOMINATOR.config_file),
				MockIO::file_write_json(THRESHOLD_DENOMINATOR.config_file, final_chain_config),
			])
		}
	}

	#[test]
	fn happy_path() {
		let final_chain_config = test_chain_config();
		let mock_context = MockIOContext::new()
			.with_file(CARDANO_CLI.default.unwrap(), "<mock executable>")
			.with_expected_io(vec![
			scenarios::show_intro(),
			MockIO::prompt(CARDANO_CLI.name, CARDANO_CLI.default, "cardano-cli"),
			MockIO::file_write_json(
				CARDANO_CLI.config_file,
				serde_json::json!({"cardano_cli": "cardano-cli"}),
			),
			prompt_and_save_to_existing_file(CARDANO_PAYMENT_VERIFICATION_KEY_FILE, "payment.vkey"),
			MockIO::run_command("cardano-cli address key-hash --payment-verification-key-file payment.vkey", "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			save_to_new_file(GOVERNANCE_AUTHORITY, "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint("Governance authority has been set to 0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint(CHAIN_ID_PROMPT),
			prompt_and_save_to_existing_file(CHAIN_ID, "0"),
			silently_fill_legacy_chain_params(final_chain_config),
            ]);

		let result = prepare_chain_params(&mock_context);

		mock_context.no_more_io_expected();
		assert!(result.is_ok());
	}

	#[test]
	fn happy_path_with_overwriting_governance_authority() {
		let final_chain_config = test_chain_config();

		let initial_chain_config = serde_json::json!({
			"chain_parameters": {
				"governance_authority": GOV_AUTH,
			}
		});

		let mock_context = MockIOContext::new()
			.with_file(CARDANO_CLI.default.unwrap(), "<mock executable>")
			.with_json_file(CHAIN_CONFIG_PATH, initial_chain_config).with_expected_io(vec![
			scenarios::show_intro(),
			MockIO::file_read(GOVERNANCE_AUTHORITY.config_file),
			MockIO::eprint(&GOVERNANCE_AUTHORITY.loaded_from_config_msg(&MainchainAddressHash::from_hex_unsafe(GOV_AUTH))),
			MockIO::prompt_yes_no(&is_gov_auth_valid_prompt(), true, false),
			MockIO::prompt(CARDANO_CLI.name, CARDANO_CLI.default, "cardano-cli"),
			MockIO::file_write_json(
                CARDANO_CLI.config_file,
                serde_json::json!({"cardano_cli": "cardano-cli"}),
            ),
			prompt_and_save_to_existing_file(CARDANO_PAYMENT_VERIFICATION_KEY_FILE, "payment.vkey"),
			MockIO::run_command("cardano-cli address key-hash --payment-verification-key-file payment.vkey", "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			save_to_existing_file(GOVERNANCE_AUTHORITY, GOV_AUTH),
			MockIO::eprint("Governance authority has been set to 0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint(CHAIN_ID_PROMPT),
			prompt_and_save_to_existing_file(CHAIN_ID, "0"),
			silently_fill_legacy_chain_params(final_chain_config),
        ]);

		let result = prepare_chain_params(&mock_context);

		mock_context.no_more_io_expected();

		assert!(result.is_ok());
	}

	#[test]
	fn happy_path_without_overwriting_governance_authority() {
		let mut final_chain_config = test_chain_config();
		if let Some(gov_auth) = final_chain_config.pointer_mut(&GOVERNANCE_AUTHORITY.json_pointer())
		{
			*gov_auth = "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9".into();
		}

		let initial_chain_config = serde_json::json!({
			"chain_parameters": {
				"governance_authority": "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9",
			}
		});

		let mock_context = MockIOContext::new()
			.with_json_file(CHAIN_CONFIG_PATH, initial_chain_config)
			.with_expected_io(vec![
				scenarios::show_intro(),
				MockIO::file_read(GOVERNANCE_AUTHORITY.config_file),
				MockIO::eprint(
					&GOVERNANCE_AUTHORITY
						.loaded_from_config_msg(&MainchainAddressHash::from_hex_unsafe(GOV_AUTH)),
				),
				MockIO::prompt_yes_no(&is_gov_auth_valid_prompt(), true, true),
				MockIO::eprint(CHAIN_ID_PROMPT),
				prompt_and_save_to_existing_file(CHAIN_ID, "0"),
				silently_fill_legacy_chain_params(final_chain_config),
			]);

		let result = prepare_chain_params(&mock_context);

		mock_context.no_more_io_expected();

		assert!(result.is_ok());
	}

	#[test]
	fn happy_path_with_default_from_config() {
		let mut final_chain_config = test_chain_config();
		if let Some(chain_id) = final_chain_config.pointer_mut(&CHAIN_ID.json_pointer()) {
			*chain_id = 2.into();
		}

		let initial_chain_config = serde_json::json!({
			"chain_parameters": {
				"chain_id": 1,
			}
		});

		let mock_context = MockIOContext::new()
			.with_file(CARDANO_CLI.default.unwrap(), "<mock executable>")
			.with_json_file(CHAIN_ID.config_file, initial_chain_config).with_expected_io(vec![
			scenarios::show_intro(),
			MockIO::file_read(GOVERNANCE_AUTHORITY.config_file),
			MockIO::prompt(CARDANO_CLI.name, CARDANO_CLI.default, "cardano-cli"),
			MockIO::file_write_json(
				CARDANO_CLI.config_file,
				serde_json::json!({"cardano_cli": "cardano-cli"}),
			),
			prompt_and_save_to_existing_file(CARDANO_PAYMENT_VERIFICATION_KEY_FILE, "payment.vkey"),
			MockIO::run_command("cardano-cli address key-hash --payment-verification-key-file payment.vkey", "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			save_to_existing_file(GOVERNANCE_AUTHORITY,  "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint("Governance authority has been set to 0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint(CHAIN_ID_PROMPT),
			prompt_with_default_and_save_to_existing_file(CHAIN_ID, Some("1"),"2"),
			silently_fill_legacy_chain_params(final_chain_config),
		]);

		let result = prepare_chain_params(&mock_context);

		mock_context.no_more_io_expected();
		assert!(result.is_ok());
	}

	#[test]
	fn keep_legacy_params_if_present_in_config() {
		let genesis_committee_utxo =
			"0000000000000000000000000000000000000000000000000000000000000000#1";
		let threshold_numerator: u64 = 4;
		let threshold_denominator: u64 = 5;

		let final_chain_config = serde_json::json!({
			"chain_parameters": {
				"governance_authority": "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9",
				"chain_id": 0,
				"genesis_committee_utxo": genesis_committee_utxo,
				"threshold_numerator": threshold_numerator,
				"threshold_denominator": threshold_denominator,
			}
		});

		let initial_chain_config = serde_json::json!({
			"chain_parameters": {
				"genesis_committee_utxo": genesis_committee_utxo,
				"threshold_numerator": threshold_numerator,
				"threshold_denominator": threshold_denominator,
			}
		});

		let mock_context = MockIOContext::new()
			.with_file(CARDANO_CLI.default.unwrap(), "<mock executable>")
			.with_json_file(GENESIS_COMMITTEE_UTXO.config_file ,initial_chain_config)
			.with_expected_io(vec![
			scenarios::show_intro(),
			MockIO::file_read(GOVERNANCE_AUTHORITY.config_file),
			MockIO::prompt(CARDANO_CLI.name, CARDANO_CLI.default, "cardano-cli"),
			MockIO::file_write_json(
				CARDANO_CLI.config_file,
				serde_json::json!({"cardano_cli": "cardano-cli"}),
			),
			prompt_and_save_to_existing_file(CARDANO_PAYMENT_VERIFICATION_KEY_FILE, "payment.vkey"),
			MockIO::run_command("cardano-cli address key-hash --payment-verification-key-file payment.vkey", "76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			save_to_existing_file(GOVERNANCE_AUTHORITY, "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint("Governance authority has been set to 0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9"),
			MockIO::eprint(CHAIN_ID_PROMPT),
			MockIO::file_read(CHAIN_ID.config_file),
			MockIO::prompt(CHAIN_ID.name, CHAIN_ID.default, "0"),
			MockIO::file_read(CHAIN_ID.config_file),
			MockIO::file_write_json(CHAIN_ID.config_file, final_chain_config),
			MockIO::file_read(GENESIS_COMMITTEE_UTXO.config_file),
			MockIO::eprint(&GENESIS_COMMITTEE_UTXO.loaded_from_config_msg(&UtxoId::from_str(genesis_committee_utxo).unwrap())),
			MockIO::file_read(THRESHOLD_NUMERATOR.config_file),
			MockIO::eprint(&THRESHOLD_NUMERATOR.loaded_from_config_msg(&threshold_numerator)),
			MockIO::file_read(THRESHOLD_DENOMINATOR.config_file),
			MockIO::eprint(&THRESHOLD_DENOMINATOR.loaded_from_config_msg(&threshold_denominator)),
		]);

		let result = prepare_chain_params(&mock_context);

		mock_context.no_more_io_expected();
		assert!(result.is_ok());
	}

	fn test_chain_config() -> Value {
		serde_json::json!({
			"chain_parameters": {
				"governance_authority": "0x76da17b2e3371ab7ca88ce0500441149f03cc5091009f99c99c080d9",
				"chain_id": 0,
				"genesis_committee_utxo": GENESIS_COMMITTEE_UTXO.default.unwrap(),
				"threshold_numerator": 2,
				"threshold_denominator": 3
			}
		})
	}
}
