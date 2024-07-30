
//! Autogenerated weights for `pallet_session_validator_management`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 32.0.0
//! DATE: 2024-05-28, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `michal-pc`, CPU: `AMD Ryzen 9 5950X 16-Core Processor`
//! WASM-EXECUTION: `Compiled`, CHAIN: `None`, DB CACHE: 1024

// Executed Command:
// ./target/production/partner-chains-node
// benchmark
// pallet
// --steps=50
// --repeat=20
// --pallet=pallet_session_validator_management
// --extrinsic=*
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtime/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_session_validator_management`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_session_validator_management::WeightInfo for WeightInfo<T> {
	/// Storage: `SessionCommitteeManagement::CurrentCommittee` (r:1 w:0)
	/// Proof: `SessionCommitteeManagement::CurrentCommittee` (`max_values`: Some(1), `max_size`: Some(3113), added: 3608, mode: `MaxEncodedLen`)
	/// Storage: `SessionCommitteeManagement::NextCommittee` (r:0 w:1)
	/// Proof: `SessionCommitteeManagement::NextCommittee` (`max_values`: Some(1), `max_size`: Some(3113), added: 3608, mode: `MaxEncodedLen`)
	/// The range of component `v` is `[0, 32]`.
	fn set(v: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `138`
		//  Estimated: `4598`
		// Minimum execution time: 6_660_000 picoseconds.
		Weight::from_parts(7_070_352, 0)
			.saturating_add(Weight::from_parts(0, 4598))
			// Standard Error: 831
			.saturating_add(Weight::from_parts(58_063, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}
