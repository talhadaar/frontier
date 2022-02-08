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

use super::*;
use frame_support::{storage::migration::put_storage_value, traits::OnRuntimeUpgrade};
use hex_literal::hex;
#[test]
fn test_migrate_current_block_to_v2() {
	let (_, mut ext) = new_test_ext(1);

	ext.execute_with(|| {
		let pallet_prefix: &[u8] = b"Ethereum";
		let storage_item_prefix: &[u8] = b"CurrentBlock";
		let transaction_v0 = ethereum::LegacyTransaction {
			nonce: U256::zero(),
			gas_price: U256::from(1),
			gas_limit: U256::from(0x100000),
			action: ethereum::TransactionAction::Create,
			value: U256::zero(),
			input: FromHex::from_hex(ERC20_CONTRACT_BYTECODE).unwrap(),
			signature: ethereum::TransactionSignature::new(
				38,
				hex!("be67e0a07db67da8d446f76add590e54b6e92cb6b8f9835aeb67540579a27717").into(),
				hex!("2d690516512020171c1ec870f6ff45398cc8609250326be89915fb538e7bd718").into(),
			)
			.unwrap(),
		};

		let ethereum_block_v0 = ethereum::BlockV0 {
			header: ethereum::Header {
				parent_hash: H256::default(),
				ommers_hash: H256::default(),
				beneficiary: H160::default(),
				state_root: H256::default(),
				transactions_root: H256::default(),
				receipts_root: H256::default(),
				logs_bloom: ethereum_types::Bloom::default(),
				difficulty: U256::default(),
				number: U256::default(),
				gas_limit: U256::default(),
				gas_used: U256::default(),
				timestamp: u64::default(),
				extra_data: vec![],
				mix_hash: H256::default(),
				nonce: ethereum_types::H64::default(),
			},
			transactions: vec![transaction_v0],
			ommers: vec![],
		};
		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&[],
			Some(ethereum_block_v0),
		);

		// We run the migration
		crate::migrations::MigrateCurrentBlockToV2::<Test>::on_runtime_upgrade();

		let current_block = Ethereum::current_block();
		assert!(current_block.is_some());
		assert_eq!(current_block.unwrap().transactions.len(), 1);
	});
}
