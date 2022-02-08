// SPDX-License-Identifier: Apache-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{Config, CurrentBlock};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::get_storage_value,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;

pub struct MigrateCurrentBlockToV2<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateCurrentBlockToV2<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		migrate_current_block_to_v2_pre_upgrade::<T, Self>()
	}
	fn on_runtime_upgrade() -> Weight {
		migrate_current_block_to_v2::<T>()
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		migrate_current_block_to_v2_post_upgrade::<T, Self>()
	}
}

// Migrates a potential CurrentBlock that is in V0 to a block in V2
fn migrate_current_block_to_v2<T: Config>() -> Weight {
	log::info!(target: "migrate_current_block_to_v2", "actually running it");
	let pallet_prefix: &[u8] = b"Ethereum";
	let storage_item_prefix: &[u8] = b"CurrentBlock";

	// Read the current block into memory.
	// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
	let stored_data =
		get_storage_value::<Option<ethereum::BlockV0>>(pallet_prefix, storage_item_prefix, &[]);
	let db_weights = T::DbWeight::get();

	let mut used_weight = db_weights.read;

	// Check if the block is V0
	if let Some(Some(current_block)) = stored_data {
		// if so, convert it to v2
		let new_block: ethereum::BlockV2 = current_block.into();
		// Update currentBlock
		CurrentBlock::<T>::put(new_block);
		// Update weight due to this branch
		used_weight = used_weight.saturating_add(db_weights.write);
	}
	used_weight
}

#[cfg(feature = "try-runtime")]
fn migrate_current_block_to_v2_pre_upgrade<T: Config, R: OnRuntimeUpgradeHelpersExt>(
) -> Result<(), &'static str> {
	let pallet_prefix: &[u8] = b"Ethereum";
	let storage_item_prefix: &[u8] = b"CurrentBlock";

	// Read the current block into memory.
	// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
	let stored_data =
		get_storage_value::<Option<ethereum::BlockV0>>(pallet_prefix, storage_item_prefix, &[]);

	if let Some(Some(current_block)) = stored_data {
		// We are gonna write some parameter of the block V0 to check later
		R::set_temp_storage(
			current_block.transactions.len() as u32,
			"transaction_length",
		);
	}
	Ok(())
}

#[cfg(feature = "try-runtime")]
fn migrate_current_block_to_v2_post_upgrade<T: Config, R: OnRuntimeUpgradeHelpersExt>(
) -> Result<(), &'static str> {
	// Check the length of the transactions matches the one written
	let old_transaction_length: Option<u32> = R::get_temp_storage("transaction_length");

	if let Some(old_transaction_length) = old_transaction_length {
		let current_block = CurrentBlock::<T>::get()
			.expect("If temp_storage exists is because we needed to migrate");
		assert_eq!(
			old_transaction_length,
			current_block.transactions.len() as u32
		);
	}
	Ok(())
}
