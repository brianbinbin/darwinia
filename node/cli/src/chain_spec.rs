// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use primitives::{ed25519, sr25519, Pair, crypto::UncheckedInto};
use node_primitives::{AccountId, AuraId};
use node_runtime::{ AuraConfig, SystemConfig,
	SessionConfig, StakingConfig, StakerStatus, TimestampConfig, BalancesConfig,
	SudoConfig, ContractsConfig, GrandpaConfig, IndicesConfig, KtonConfig, Permill, Perbill, SessionKeys};
pub use node_runtime::GenesisConfig;
use srml_support::traits::Get;
// custom
pub use node_runtime::ErasPerEpoch;
use substrate_service;
use hex_literal::hex;
use substrate_telemetry::TelemetryEndpoints;
use grandpa::AuthorityId as GrandpaId;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const TELEMETRY_URL: &str = "ws://telemetry.polkadot.io:1024";

/// Specialized `ChainSpec`.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_embedded(include_bytes!("../res/flaming-fir.json"))
}

pub fn darwinia_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_embedded(include_bytes!("../res/darwinia-fir.json"))
}

pub fn trilobita_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Darwinia POC-1 Testnet",
		"darwinia_poc_1_testnet",
		darwinia_poc1_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)])),
		None,
		None,
		None)
}

fn darwinia_poc1_testnet_config_genesis() -> GenesisConfig {
	let initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId)> =
		vec![
			get_authority_keys_from_seed("Alice"),
			get_authority_keys_from_seed("Bob"),
//			(
//			// alex
//			hex!["c6856f9b0c3ffcc5d8a19a8d545b031f6fdcd480bc4a776f5c4737973ef3de3b"].unchecked_into(),
//			hex!["3861d67edcb1950906591ecc33ade54b3aa28ca6dcc36fb025169351a8aea024"].unchecked_into(),
//			hex!["5ebec212caea5988c173ef4f0abcd9d2649711f3f5417b3c50d840576451e88c"].unchecked_into(),
//			hex!["5ebec212caea5988c173ef4f0abcd9d2649711f3f5417b3c50d840576451e88c"].unchecked_into(),
//			),(
//			// yak
//			hex!["9453279ee3b21d3f4b23f9ce168f88042dee6662a5f0673bac1cf636aaaf7f19"].unchecked_into(),
//			hex!["62bb1c167bc3f8b560c52911b5eb32b4b0249bb1d8540d1564c4344554fe1752"].unchecked_into(),
//			hex!["c621d9fb2d23140f80f4ad511b62e4677d367c56e54abc184946f769ab4382b0"].unchecked_into(),
//			hex!["c621d9fb2d23140f80f4ad511b62e4677d367c56e54abc184946f769ab4382b0"].unchecked_into(),
//			)
	];

	let endowed_accounts: Vec<AccountId> = vec![
		get_account_id_from_seed("Alice"),
		get_account_id_from_seed("Bob"),
		// jane
		hex!["d291fa464b54dccbc02b2098b8e479b567fe183609baa2c42636b73cc4eed535"].unchecked_into(),
		// t
		hex!["2e842a5fc9bacedb87d8a3db914825f7da57f5a5149170802d47e52c11109647"].unchecked_into(),
		// wb
		hex!["1486518478c79befe09ffe69dc1eb8cb862e29ee013097b021fafcb74642127b"].unchecked_into(),

		// alex
		hex!["c6856f9b0c3ffcc5d8a19a8d545b031f6fdcd480bc4a776f5c4737973ef3de3b"].unchecked_into(),
		hex!["3861d67edcb1950906591ecc33ade54b3aa28ca6dcc36fb025169351a8aea024"].unchecked_into(),
		//yak
		hex!["9453279ee3b21d3f4b23f9ce168f88042dee6662a5f0673bac1cf636aaaf7f19"].unchecked_into(),
		hex!["62bb1c167bc3f8b560c52911b5eb32b4b0249bb1d8540d1564c4344554fe1752"].unchecked_into(),
		// root
		hex!["eaf872344d3158f575ba2ffd38b5c8630955d53a74d1c845939da71b6aa06133"].unchecked_into(),
	];

	testnet_genesis(
		initial_authorities,
		hex!["eaf872344d3158f575ba2ffd38b5c8630955d53a74d1c845939da71b6aa06133"].unchecked_into(),
		Some(endowed_accounts),
		false
	)
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	// stash, controller, session-key
	// generated with secret:
	// for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/fir/$j/$i; done; done
	// and
	// for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//fir//$j//$i; done; done

	let initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId)> = vec![(
		// 5Fbsd6WXDGiLTxunqeK5BATNiocfCqu9bS1yArVjCgeBLkVy
		hex!["9c7a2ee14e565db0c69f78c7b4cd839fbf52b607d867e9e9c5a79042898a0d12"].unchecked_into(),
		// 5EnCiV7wSHeNhjW3FSUwiJNkcc2SBkPLn5Nj93FmbLtBjQUq
		hex!["781ead1e2fa9ccb74b44c19d29cb2a7a4b5be3972927ae98cd3877523976a276"].unchecked_into(),
		// 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
		hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"].unchecked_into(),
		// 5Fb9ayurnxnaXj56CjmyQLBiadfRCqUbL2VWNbbe1nZU6wiC
		hex!["9becad03e6dcac03cee07edebca5475314861492cdfc96a2144a67bbe9699332"].unchecked_into(),
	),(
		// 5ERawXCzCWkjVq3xz1W5KGNtVx2VdefvZ62Bw1FEuZW4Vny2
		hex!["68655684472b743e456907b398d3a44c113f189e56d1bbfd55e889e295dfde78"].unchecked_into(),
		// 5Gc4vr42hH1uDZc93Nayk5G7i687bAQdHHc9unLuyeawHipF
		hex!["c8dc79e36b29395413399edaec3e20fcca7205fb19776ed8ddb25d6f427ec40e"].unchecked_into(),
		// 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
		hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"].unchecked_into(),
		// 5EockCXN6YkiNCDjpqqnbcqd4ad35nU4RmA1ikM4YeRN4WcE
		hex!["7932cff431e748892fa48e10c63c17d30f80ca42e4de3921e641249cd7fa3c2f"].unchecked_into(),
	),(
		// 5DyVtKWPidondEu8iHZgi6Ffv9yrJJ1NDNLom3X9cTDi98qp
		hex!["547ff0ab649283a7ae01dbc2eb73932eba2fb09075e9485ff369082a2ff38d65"].unchecked_into(),
		// 5FeD54vGVNpFX3PndHPXJ2MDakc462vBCD5mgtWRnWYCpZU9
		hex!["9e42241d7cd91d001773b0b616d523dd80e13c6c2cab860b1234ef1b9ffc1526"].unchecked_into(),
		// 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
		hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"].unchecked_into(),
		// 5E1jLYfLdUQKrFrtqoKgFrRvxM3oQPMbf6DfcsrugZZ5Bn8d
		hex!["5633b70b80a6c8bb16270f82cca6d56b27ed7b76c8fd5af2986a25a4788ce440"].unchecked_into(),
	),(
		// 5HYZnKWe5FVZQ33ZRJK1rG3WaLMztxWrrNDb1JRwaHHVWyP9
		hex!["f26cdb14b5aec7b2789fd5ca80f979cef3761897ae1f37ffb3e154cbcc1c2663"].unchecked_into(),
		// 5EPQdAQ39WQNLCRjWsCk5jErsCitHiY5ZmjfWzzbXDoAoYbn
		hex!["66bc1e5d275da50b72b15de072a2468a5ad414919ca9054d2695767cf650012f"].unchecked_into(),
		// 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
		hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"].unchecked_into(),
		// 5DMa31Hd5u1dwoRKgC4uvqyrdK45RHv3CpwvpUC1EzuwDit4
		hex!["3919132b851ef0fd2dae42a7e734fe547af5a6b809006100f48944d7fae8e8ef"].unchecked_into(),
	)];

	// generated with secret: subkey inspect "$secret"/fir
	let endowed_accounts: Vec<AccountId> = vec![
		// 5Ff3iXP75ruzroPWRP2FYBHWnmGGBSb63857BgnzCoXNxfPo
		hex!["9ee5e5bdc0ec239eb164f865ecc345ce4c88e76ee002e0f7e318097347471809"].unchecked_into(),
	];

	const MILLICENTS: u128 = 1_000_000_000;
	const CENTS: u128 = 1_000 * MILLICENTS;    // assume this is worth about a cent.
	const DOLLARS: u128 = 100 * CENTS;

	const SECS_PER_BLOCK: u64 = 6;
	const MINUTES: u64 = 60 / SECS_PER_BLOCK;
	const HOURS: u64 = MINUTES * 60;
	const DAYS: u64 = HOURS * 24;

	const ENDOWMENT: u128 = 10_000_000 * DOLLARS;
	const STASH: u128 = 100 * DOLLARS;
	const TEN: u128 = 10_000;

	GenesisConfig {
		system: Some(SystemConfig {
			code: include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm").to_vec(),    // FIXME change once we have #1252
			_genesis_phantom_data: Default::default(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1 * CENTS,
			transaction_byte_fee: 10 * MILLICENTS,
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
			existential_deposit: 1 * DOLLARS,
			transfer_fee: 1 * CENTS,
			creation_fee: 1 * CENTS,
			vesting: vec![],
		}),
//		ring: Some(RingConfig {
//			transaction_base_fee: 1 * CENTS,
//			transaction_byte_fee: 10 * MILLICENTS,
//			balances: endowed_accounts.iter().cloned()
//				.map(|k| (k, ENDOWMENT))
//				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
//				.collect(),
//			existential_deposit: 0,
//			transfer_fee: 1 * CENTS,
//			creation_fee: 1 * CENTS,
//			vesting: vec![],
//		}),
		kton: Some(KtonConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, TEN))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), TEN)))
				.collect(),
			vesting: vec![],
			sys_acc: hex!["984d592d15d930ac36e6716407fbed3f7d1e2e62bc11f8429345f8b8b0dfc107"].unchecked_into(),
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.iter().cloned()
				.chain(initial_authorities.iter().map(|x| x.0.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			keys: initial_authorities.iter().map(|x| (x.1.clone(), SessionKeys(x.2.clone(),x.2.clone()))).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			epoch_index: 0,
			current_era_total_reward: 1_600_000_000_000 / ErasPerEpoch::get() as u128,
			offline_slash: Perbill::from_parts(1_000_000),
			session_reward: Perbill::from_percent(40),
			current_session_reward: 0,
			validator_count: 7,
			offline_slash_grace: 4,
			minimum_validator_count: 4,
			stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
		}),
//		democracy: Some(DemocracyConfig::default()),
//		council_seats: Some(CouncilSeatsConfig {
//			active_council: vec![],
//			candidacy_bond: 10 * DOLLARS,
//			voter_bond: 1 * DOLLARS,
//			voting_fee: 2 * DOLLARS,
//			present_slash_per_voter: 1 * CENTS,
//			carry_count: 6,
//			presentation_duration: 1 * DAYS,
//			approval_voting_period: 2 * DAYS,
//			term_duration: 28 * DAYS,
//			desired_seats: 0,
//			decay_ratio: 0,
//			inactive_grace_period: 1,    // one additional vote should go by before an inactive voter can be reaped.
//		}),
		timestamp: Some(TimestampConfig {
			minimum_period: SECS_PER_BLOCK / 2, // due to the nature of aura the slots are 2*period
		}),
//		treasury: Some(TreasuryConfig {
//			proposal_bond: Permill::from_percent(5),
//			proposal_bond_minimum: 1 * DOLLARS,
//			spend_period: 1 * DAYS,
//			burn: Permill::from_percent(50),
//		}),
		contracts: Some(ContractsConfig {
			signed_claim_handicap: 2,
			rent_byte_price: 4,
			rent_deposit_offset: 1000,
			storage_size_offset: 8,
			surcharge_reward: 150,
			tombstone_deposit: 16,
			transaction_base_fee: 1 * CENTS,
			transaction_byte_fee: 10 * MILLICENTS,
			transfer_fee: 1 * CENTS,
			creation_fee: 1 * CENTS,
			contract_fee: 1 * CENTS,
			call_base_fee: 1000,
			create_base_fee: 1000,
			gas_price: 1 * MILLICENTS,
			max_depth: 1024,
			block_gas_limit: 10_000_000,
			current_schedule: Default::default(),
		}),
		sudo: Some(SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.3.clone(), 1)).collect(),
			_genesis_phantom_data: Default::default(),
		}),
	}
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		None,
		None,
		None,
	)
}

/// Helper function to generate AccountId from seed
pub fn get_account_id_from_seed(seed: &str) -> AccountId {
	sr25519::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate AuraId from seed
pub fn get_aura_id_from_seed(seed: &str) -> AuraId {
	ed25519::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate GrandpaId from seed
pub fn get_grandpa_id_from_seed(seed: &str) -> GrandpaId {
	ed25519::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed(&format!("{}//stash", seed)),
		get_account_id_from_seed(seed),
		get_aura_id_from_seed(seed),
		get_grandpa_id_from_seed(seed)
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	enable_println: bool,
) -> GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed("Alice"),
			get_account_id_from_seed("Bob"),
			get_account_id_from_seed("Charlie"),
			get_account_id_from_seed("Dave"),
			get_account_id_from_seed("Eve"),
			get_account_id_from_seed("Ferdie"),
			get_account_id_from_seed("Alice//stash"),
			get_account_id_from_seed("Bob//stash"),
			get_account_id_from_seed("Charlie//stash"),
			get_account_id_from_seed("Dave//stash"),
			get_account_id_from_seed("Eve//stash"),
			get_account_id_from_seed("Ferdie//stash"),
		]
	});

	const MILLICENTS: u128 = 1_000_000_000;
	const CENTS: u128 = 1_000 * MILLICENTS;    // assume this is worth about a cent.
	const DOLLARS: u128 = 100 * CENTS;

	const SECS_PER_BLOCK: u64 = 6;
	const MINUTES: u64 = 60 / SECS_PER_BLOCK;
	const HOURS: u64 = MINUTES * 60;
	const DAYS: u64 = HOURS * 24;

	const STASH: u128 = 1 << 20;
	const ENDOWMENT: u128 = 1 << 20;
	const TEN: u128 = 10_000;

	let council_desired_seats = (endowed_accounts.len() / 2 - initial_authorities.len()) as u32;
	let mut contracts_config = ContractsConfig {
		signed_claim_handicap: 2,
		rent_byte_price: 4,
		rent_deposit_offset: 1000,
		storage_size_offset: 8,
		surcharge_reward: 150,
		tombstone_deposit: 16,
		transaction_base_fee: 1,
		transaction_byte_fee: 0,
		transfer_fee: 0,
		creation_fee: 0,
		contract_fee: 21,
		call_base_fee: 135,
		create_base_fee: 175,
		gas_price: 1,
		max_depth: 1024,
		block_gas_limit: 10_000_000,
		current_schedule: Default::default(),
	};
	// this should only be enabled on development chains
	contracts_config.current_schedule.enable_println = enable_println;


	GenesisConfig {
		system: Some(SystemConfig {
			code: include_bytes!("../../runtime/wasm/target/wasm32-unknown-unknown/release/node_runtime.compact.wasm").to_vec(),
			_genesis_phantom_data: Default::default(),
			changes_trie_config: Default::default(),
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 0,
			transaction_byte_fee: 0,
			existential_deposit: 1,
			transfer_fee: 1,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, 180_000_000_000))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), 100_000_000_000)))
				.collect(),
			vesting: vec![],
		}),
//		ring: Some(RingConfig {
//			transaction_base_fee: 0,
//			transaction_byte_fee: 0,
//			balances: endowed_accounts.iter().cloned()
//				.map(|k| (k, 400_000_000_000))
//				.chain(initial_authorities.iter().map(|x| (x.0.clone(), 200_000_000_000)))
//				.collect(),
//			existential_deposit: 0,
//			transfer_fee: 0,
//			creation_fee: 0,
//			vesting: vec![],
//		}),
		kton: Some(KtonConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, TEN))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), TEN)))
				.collect(),
			vesting: vec![],
			sys_acc: hex!["984d592d15d930ac36e6716407fbed3f7d1e2e62bc11f8429345f8b8b0dfc107"].unchecked_into(),
		}),
		session: Some(SessionConfig {
			validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			keys: initial_authorities.iter().map(|x| (x.1.clone(), SessionKeys(x.2.clone(), x.2.clone()))).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			epoch_index: 0,
			current_era_total_reward: 1_600_000_000_000 / ErasPerEpoch::get() as u128,
			minimum_validator_count: 1,
			validator_count: 20,
			offline_slash: Perbill::zero(),
			session_reward: Perbill::from_percent(45),
			current_session_reward: 0,
			offline_slash_grace: 0,
			stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
		}),
//		democracy: Some(DemocracyConfig::default()),
//		council_seats: Some(CouncilSeatsConfig {
//			active_council: endowed_accounts.iter()
//				.filter(|&endowed| initial_authorities.iter().find(|&(_, controller, ..)| controller == endowed).is_none())
//				.map(|a| (a.clone(), 1000000)).collect(),
//			candidacy_bond: 10,
//			voter_bond: 2,
//			voting_fee: 5,
//			present_slash_per_voter: 1,
//			carry_count: 4,
//			presentation_duration: 10,
//			approval_voting_period: 20,
//			term_duration: 1000000,
//			desired_seats: council_desired_seats,
//			decay_ratio: council_desired_seats / 3,
//			inactive_grace_period: 1,
//		}),
		timestamp: Some(TimestampConfig {
			minimum_period: 3,                    // 3*2=6 second block time.
		}),
//		treasury: Some(TreasuryConfig {
//			proposal_bond: Permill::from_percent(5),
//			proposal_bond_minimum: 1_000_000,
//			spend_period: 12 * 60 * 24,
//			burn: Permill::from_percent(50),
//		}),
		contracts: Some(contracts_config),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.3.clone(), 1)).collect(),
			_genesis_phantom_data: Default::default(),
		}),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			get_authority_keys_from_seed("Alice"),
		],
		get_account_id_from_seed("Alice"),
		None,
		true,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis("Development", "dev", development_config_genesis, vec![], None, None, None, None)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			get_authority_keys_from_seed("Alice"),
			get_authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed("Alice"),
		None,
		false,
	)
}

pub fn trilobita_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Darwinia POC-1 Testnet",
		"darwinia_poc_1_testnet",
		local_testnet_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)])),
		None,
		None,
		None)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis("Local Testnet", "local_testnet", local_testnet_genesis, vec![], None, None, None, None)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use service_test;
	use crate::service::Factory;

	fn local_testnet_genesis_instant() -> GenesisConfig {
		let mut genesis = local_testnet_genesis();
		genesis.timestamp = Some(TimestampConfig { minimum_period: 1 });
		genesis
	}

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		let mut genesis = testnet_genesis(
			vec![
				get_authority_keys_from_seed("Alice"),
			],
			get_account_id_from_seed("Alice"),
			None,
			false,
		);
		genesis.timestamp = Some(TimestampConfig { minimum_period: 1 });
		genesis
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			None,
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis("Integration Test", "test", local_testnet_genesis_instant, vec![], None, None, None, None)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		service_test::connectivity::<Factory>(integration_test_config_with_two_authorities());
	}
}
