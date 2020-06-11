// This file is part of Substrate.

// Copyright (C) 2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Client fixed chain specification rules

use std::collections::{HashMap, HashSet};

use sp_runtime::{
	traits::{Block as BlockT, NumberFor},
};

use sc_client_api::{ForkBlocks, BadBlocks};

/// Chain specification rules lookup result.
pub enum LookupResult<B: BlockT> {
	/// Specification rules do not contain any special rules about this block
	NotSpecial,
	/// The block is known to be bad and should not be imported
	KnownBad,
	/// The block is known not to be finalized
	KnownUnfinalized,
	/// There is a specified canonical block hash for the given height
	Expected(B::Hash)
}

impl<B: BlockT> LookupResult<B> {
	/// Whether the result indicates a block that should not be finalized.
	pub fn is_unfinalized(&self) -> bool {
		match self {
			Self::KnownUnfinalized => true,
			_ => false,
		}
	}
}

/// Chain-specific block filtering rules.
///
/// This holds known bad blocks and known good forks, and
/// is usually part of the chain spec.
pub struct BlockRules<B: BlockT> {
	bad: HashSet<B::Hash>,
	unfinalized: HashSet<B::Hash>,
	forks: HashMap<NumberFor<B>, B::Hash>,
}

impl<B: BlockT> BlockRules<B> {
	/// New block rules with provided black and white lists.
	pub fn new(
		fork_blocks: ForkBlocks<B>,
		bad_blocks: BadBlocks<B>,
	) -> Self {
		Self {
			bad: bad_blocks.unwrap_or(HashSet::new()),
			forks: fork_blocks.unwrap_or(vec![]).into_iter().collect(),
			unfinalized: HashSet::new(),
		}
	}

	/// Mark a block as not possible to be finalized.
	pub fn mark_unfinalized(&mut self, hash: B::Hash) {
		self.unfinalized.insert(hash);
	}

	/// Check if there's any rule affecting the given block.
	pub fn lookup(&self, number: NumberFor<B>, hash: &B::Hash) -> LookupResult<B> {
		if let Some(hash_for_height) = self.forks.get(&number) {
			if hash_for_height != hash {
				return LookupResult::Expected(hash_for_height.clone());
			}
		}

		if self.bad.contains(hash) {
			return LookupResult::KnownBad
		}

		if self.unfinalized.contains(hash) {
			return LookupResult::KnownUnfinalized
		}

		LookupResult::NotSpecial
	}

	/// Check if there's any rule affecting the given block hash.
	pub fn lookup_hash(&self, hash: &B::Hash) -> LookupResult<B> {
		if self.bad.contains(hash) {
			return LookupResult::KnownBad
		}

		if self.unfinalized.contains(hash) {
			return LookupResult::KnownUnfinalized
		}

		LookupResult::NotSpecial
	}
}
