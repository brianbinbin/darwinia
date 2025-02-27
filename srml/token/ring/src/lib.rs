// Copyright 2017-2019 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use rstd::{cmp, result, mem, convert::Into};
use parity_codec::{Codec, Encode, Decode};
use srml_support::{StorageValue, StorageMap, Parameter, decl_event, decl_storage, decl_module};
use srml_support::traits::{
	UpdateBalanceOutcome, Currency, OnFreeBalanceZero, MakePayment, OnUnbalanced,
	WithdrawReason, WithdrawReasons, LockIdentifier, LockableCurrency, ExistenceRequirement,
	Imbalance, SignedImbalance, ReservableCurrency
};
use srml_support::dispatch::Result;
use primitives::traits::{
	Zero, SimpleArithmetic, StaticLookup, Member, CheckedAdd, CheckedSub,
	MaybeSerializeDebug, Saturating
};
use system::{IsDeadAccount, OnNewAccount, ensure_signed};

mod mock;

pub use self::imbalances::{PositiveImbalance, NegativeImbalance};

pub trait Subtrait<I: Instance = DefaultInstance>: timestamp::Trait {
	/// The balance of an account.
	type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy +
		MaybeSerializeDebug + From<Self::Moment>;

	/// A function that is invoked when the free-balance has fallen below the existential deposit and
	/// has been reduced to zero.
	///
	/// Gives a chance to clean up resources associated with the given account.
	type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

	/// Handler for when a new account is created.
	type OnNewAccount: OnNewAccount<Self::AccountId>;
}

pub trait Trait<I: Instance = DefaultInstance>: timestamp::Trait {
	/// The balance of an account.
	type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy +
		MaybeSerializeDebug + From<Self::Moment>;

	/// A function that is invoked when the free-balance has fallen below the existential deposit and
	/// has been reduced to zero.
	///
	/// Gives a chance to clean up resources associated with the given account.
	type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

	/// Handler for when a new account is created.
	type OnNewAccount: OnNewAccount<Self::AccountId>;

	/// Handler for the unbalanced reduction when taking transaction fees.
	type TransactionPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

	/// Handler for the unbalanced reduction when taking fees associated with balance
	/// transfer (which may also include account creation).
	type TransferPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

	/// Handler for the unbalanced reduction when removing a dust account.
	type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;

	/// The overarching event type.
	type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

impl<T: Trait<I>, I: Instance> Subtrait<I> for T {
	type Balance = T::Balance;
	type OnFreeBalanceZero = T::OnFreeBalanceZero;
	type OnNewAccount = T::OnNewAccount;
}

decl_event!(
	pub enum Event<T, I: Instance = DefaultInstance> where
		<T as system::Trait>::AccountId,
		<T as Trait<I>>::Balance
	{
		/// A new account was created.
		NewAccount(AccountId, Balance),
		/// An account was reaped.
		ReapedAccount(AccountId),
		/// Transfer succeeded (from, to, value, fees).
		Transfer(AccountId, AccountId, Balance, Balance),
	}
);

/// Struct to encode the vesting schedule of an individual account.
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct VestingSchedule<Balance> {
	/// Locked amount at genesis.
	pub offset: Balance,
	/// Amount that gets unlocked every block from genesis.
	pub per_block: Balance,
}

impl<Balance: SimpleArithmetic + Copy> VestingSchedule<Balance> {
	/// Amount locked at block `n`.
	pub fn locked_at<Moment>(&self, n: Moment) -> Balance
		where Balance: From<Moment>
	{
		if let Some(x) = Balance::from(n).checked_mul(&self.per_block) {
			self.offset.max(x) - x
		} else {
			Zero::zero()
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BalanceLock<Balance, Moment> {
	pub id: LockIdentifier,
	pub amount: Balance,
	pub until: Moment,
	pub reasons: WithdrawReasons,
}

decl_storage! {
	trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as RingBalances {
		/// The total units issued in the system.
		pub TotalIssuance get(total_issuance) build(|config: &GenesisConfig<T, I>| {
			config.balances.iter().fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
		}): T::Balance;
		/// The minimum amount required to keep an account open.
		pub ExistentialDeposit get(existential_deposit) config(): T::Balance;
		/// The fee required to make a transfer.
		pub TransferFee get(transfer_fee) config(): T::Balance;
		/// The fee required to create an account.
		pub CreationFee get(creation_fee) config(): T::Balance;
		/// The fee to be paid for making a transaction; the base.
		pub TransactionBaseFee get(transaction_base_fee) config(): T::Balance;
		/// The fee to be paid for making a transaction; the per-byte portion.
		pub TransactionByteFee get(transaction_byte_fee) config(): T::Balance;

		/// Information regarding the vesting of a given account.
		pub Vesting get(vesting) : map T::AccountId => Option<VestingSchedule<T::Balance>>;

		/// The 'free' balance of a given account.
		///
		/// This is the only balance that matters in terms of most operations on tokens. It
		/// alone is used to determine the balance when in the contract execution environment. When this
		/// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
		/// deleted: specifically `FreeBalance`. Further, the `OnFreeBalanceZero` callback
		/// is invoked, giving a chance to external modules to clean up data associated with
		/// the deleted account.
		///
		/// `system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
		/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
		pub FreeBalance get(free_balance) build(|config: &GenesisConfig<T, I>| config.balances.clone()): map T::AccountId => T::Balance;

		/// The amount of the balance of a given account that is externally reserved; this can still get
		/// slashed, but gets slashed last of all.
		///
		/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
		/// that are still 'owned' by the account holder, but which are suspendable.
		///
		/// When this balance falls below the value of `ExistentialDeposit`, then this 'reserve account'
		/// is deleted: specifically, `ReservedBalance`.
		///
		/// `system::AccountNonce` is also deleted if `FreeBalance` is also zero (it also gets
		/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.)
		pub ReservedBalance get(reserved_balance): map T::AccountId => T::Balance;

		/// Any liquidity locks on some account ring.
		pub Locks get(locks): map T::AccountId => Vec<BalanceLock<T::Balance, T::Moment>>;
	}
	add_extra_genesis {
		config(balances): Vec<(T::AccountId, T::Balance)>;
		config(vesting): Vec<(T::AccountId, u64, u64)>;		// begin, length
	}
	extra_genesis_skip_phantom_data_field;
}

decl_module! {
	pub struct Module<T: Trait<I>, I: Instance = DefaultInstance> for enum Call where origin: T::Origin {
		fn deposit_event<T, I>() = default;

		/// Transfer some liquid free balance to another account.
		///
		/// `transfer` will set the `FreeBalance` of the sender and receiver.
		/// It will decrease the total issuance of the system by the `TransferFee`.
		/// If the sender's account is below the existential deposit as a result
		/// of the transfer, the account will be reaped.
		///
		/// The dispatch origin for this call must be `Signed` by the transactor.
		///
		/// # <weight>
		/// - Dependent on arguments but not critical, given proper implementations for
		///   input config types. See related functions below.
		/// - It contains a limited number of reads and writes internally and no complex computation.
		///
		/// Related functions:
		///
		///   - `ensure_can_withdraw` is always called internally but has a bounded complexity.
		///   - Transferring ring to accounts that did not exist before will cause
		///      `T::OnNewAccount::on_new_account` to be called.
		///   - Removing enough funds from an account will trigger
		///     `T::DustRemoval::on_unbalanced` and `T::OnFreeBalanceZero::on_free_balance_zero`.
		///
		/// # </weight>
		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&transactor, &dest, value)?;
		}

		/// Set the ring of a given account.
		///
		/// This will alter `FreeBalance` and `ReservedBalance` in storage. it will
		/// also decrease the total issuance of the system (`TotalIssuance`).
		/// If the new free or reserved balance is below the existential deposit,
		/// it will reset the account nonce (`system::AccountNonce`).
		///
		/// The dispatch origin for this call is `root`.
		///
		/// # <weight>
		/// - Independent of the arguments.
		/// - Contains a limited number of reads and writes.
		/// # </weight>
		fn set_balance(
			who: <T::Lookup as StaticLookup>::Source,
			#[compact] new_free: T::Balance,
			#[compact] new_reserved: T::Balance
		) {
			let who = T::Lookup::lookup(who)?;

			let current_free = <FreeBalance<T, I>>::get(&who);
			if new_free > current_free {
				mem::drop(PositiveImbalance::<T, I>::new(new_free - current_free));
			} else if new_free < current_free {
				mem::drop(NegativeImbalance::<T, I>::new(current_free - new_free));
			}
			Self::set_free_balance(&who, new_free);

			let current_reserved = <ReservedBalance<T, I>>::get(&who);
			if new_reserved > current_reserved {
				mem::drop(PositiveImbalance::<T, I>::new(new_reserved - current_reserved));
			} else if new_reserved < current_reserved {
				mem::drop(NegativeImbalance::<T, I>::new(current_reserved - new_reserved));
			}
			Self::set_reserved_balance(&who, new_reserved);
		}
	}
}

impl<T: Trait<I>, I: Instance> Module<T, I> {

	// PUBLIC IMMUTABLES

	/// Get the amount that is currently being vested and cannot be transferred out of this account.
	pub fn vesting_balance(who: &T::AccountId) -> T::Balance {
		if let Some(v) = Self::vesting(who) {
			Self::free_balance(who)
				.min(v.locked_at::<T::Moment>(<timestamp::Module<T>>::now()))
		} else {
			Zero::zero()
		}
	}

	// PRIVATE MUTABLES

	/// Set the reserved balance of an account to some new value. Will enforce `ExistentialDeposit`
	/// law, annulling the account as needed.
	///
	/// Doesn't do any preparatory work for creating a new account, so should only be used when it
	/// is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_reserved_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
		if balance < Self::existential_deposit() {
			<ReservedBalance<T, I>>::insert(who, balance);
			Self::on_reserved_too_low(who);
			UpdateBalanceOutcome::AccountKilled
		} else {
			<ReservedBalance<T, I>>::insert(who, balance);
			UpdateBalanceOutcome::Updated
		}
	}

	/// Set the free balance of an account to some new value. Will enforce `ExistentialDeposit`
	/// law, annulling the account as needed.
	///
	/// Doesn't do any preparatory work for creating a new account, so should only be used when it
	/// is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn set_free_balance(who: &T::AccountId, balance: T::Balance) -> UpdateBalanceOutcome {
		// Commented out for now - but consider it instructive.
		// assert!(!Self::total_balance(who).is_zero());
		// assert!(Self::free_balance(who) > Self::existential_deposit());
		if balance < Self::existential_deposit() {
			<FreeBalance<T, I>>::insert(who, balance);
			Self::on_free_too_low(who);
			UpdateBalanceOutcome::AccountKilled
		} else {
			<FreeBalance<T, I>>::insert(who, balance);
			UpdateBalanceOutcome::Updated
		}
	}

	/// Register a new account (with existential balance).
	///
	/// This just calls appropriate hooks. It doesn't (necessarily) make any state changes.
	fn new_account(who: &T::AccountId, balance: T::Balance) {
		T::OnNewAccount::on_new_account(&who);
		Self::deposit_event(RawEvent::NewAccount(who.clone(), balance.clone()));
	}

	/// Unregister an account.
	///
	/// This just removes the nonce and leaves an event.
	fn reap_account(who: &T::AccountId) {
		<system::AccountNonce<T>>::remove(who);
		Self::deposit_event(RawEvent::ReapedAccount(who.clone()));
	}

	/// Account's free balance has dropped below existential deposit. Kill its
	/// free side and the account completely if its reserved size is already dead.
	///
	/// Will maintain total issuance.
	fn on_free_too_low(who: &T::AccountId) {
		let dust = <FreeBalance<T, I>>::take(who);
		<Locks<T, I>>::remove(who);

		// underflow should never happen, but if it does, there's not much we can do about it.
		if !dust.is_zero() {
			T::DustRemoval::on_unbalanced(NegativeImbalance::new(dust));
		}

		T::OnFreeBalanceZero::on_free_balance_zero(who);

		if Self::reserved_balance(who).is_zero() {
			Self::reap_account(who);
		}
	}

	/// Account's reserved balance has dropped below existential deposit. Kill its
	/// reserved side and the account completely if its free size is already dead.
	///
	/// Will maintain total issuance.
	fn on_reserved_too_low(who: &T::AccountId) {
		let dust = <ReservedBalance<T, I>>::take(who);

		// underflow should never happen, but it if does, there's nothing to be done here.
		if !dust.is_zero() {
			T::DustRemoval::on_unbalanced(NegativeImbalance::new(dust));
		}

		if Self::free_balance(who).is_zero() {
			Self::reap_account(who);
		}
	}
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
	use super::{
		result, Subtrait, DefaultInstance, Imbalance, Trait, Zero, Instance, Saturating,
		StorageValue,
	};
	use rstd::mem;

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been created without any equal and opposite accounting.
	#[must_use]
	pub struct PositiveImbalance<T: Subtrait<I>, I: Instance=DefaultInstance>(T::Balance);

	impl<T: Subtrait<I>, I: Instance> PositiveImbalance<T, I> {
		/// Create a new positive imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			PositiveImbalance(amount)
		}
	}

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been destroyed without any equal and opposite accounting.
	#[must_use]
	pub struct NegativeImbalance<T: Subtrait<I>, I: Instance=DefaultInstance>(T::Balance);

	impl<T: Subtrait<I>, I: Instance> NegativeImbalance<T, I> {
		/// Create a new negative imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			NegativeImbalance(amount)
		}
	}

	impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for PositiveImbalance<T, I> {
		type Opposite = NegativeImbalance<T, I>;

		fn zero() -> Self {
			Self(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self(first), Self(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self(a - b))
			} else {
				Err(NegativeImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T: Trait<I>, I: Instance> Imbalance<T::Balance> for NegativeImbalance<T, I> {
		type Opposite = PositiveImbalance<T, I>;

		fn zero() -> Self {
			Self(Zero::zero())
		}
		fn drop_zero(self) -> result::Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self(first), Self(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self(a - b))
			} else {
				Err(PositiveImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T: Subtrait<I>, I: Instance> Drop for PositiveImbalance<T, I> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(
				|v| *v = v.saturating_add(self.0)
			);
		}
	}

	impl<T: Subtrait<I>, I: Instance> Drop for NegativeImbalance<T, I> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<super::TotalIssuance<super::ElevatedTrait<T, I>, I>>::mutate(
				|v| *v = v.saturating_sub(self.0)
			);
		}
	}
}

// TODO: #2052
// Somewhat ugly hack in order to gain access to module's `increase_total_issuance_by`
// using only the Subtrait (which defines only the types that are not dependent
// on Positive/NegativeImbalance). Subtrait must be used otherwise we end up with a
// circular dependency with Trait having some types be dependent on PositiveImbalance<Trait>
// and PositiveImbalance itself depending back on Trait for its Drop impl (and thus
// its type declaration).
// This works as long as `increase_total_issuance_by` doesn't use the Imbalance
// types (basically for charging fees).
// This should eventually be refactored so that the three type items that do
// depend on the Imbalance type (TransactionPayment, TransferPayment, DustRemoval)
// are placed in their own SRML module.
struct ElevatedTrait<T: Subtrait<I>, I: Instance>(T, I);
impl<T: Subtrait<I>, I: Instance> Clone for ElevatedTrait<T, I> {
	fn clone(&self) -> Self { unimplemented!() }
}
impl<T: Subtrait<I>, I: Instance> PartialEq for ElevatedTrait<T, I> {
	fn eq(&self, _: &Self) -> bool { unimplemented!() }
}
impl<T: Subtrait<I>, I: Instance> Eq for ElevatedTrait<T, I> {}
impl<T: Subtrait<I>, I: Instance> system::Trait for ElevatedTrait<T, I> {
	type Origin = T::Origin;
	type Index = T::Index;
	type BlockNumber = T::BlockNumber;
	type Hash = T::Hash;
	type Hashing = T::Hashing;
	type AccountId = T::AccountId;
	type Lookup = T::Lookup;
	type Header = T::Header;
	type Event = ();
}

impl<T: Subtrait<I>, I: Instance> timestamp::Trait for ElevatedTrait<T, I> {
	type Moment = T::Moment;
	type OnTimestampSet = ();
}

impl<T: Subtrait<I>, I: Instance> Trait<I> for ElevatedTrait<T, I> {
	type Balance = T::Balance;
	type OnFreeBalanceZero = T::OnFreeBalanceZero;
	type OnNewAccount = T::OnNewAccount;
	type Event = ();
	type TransactionPayment = ();
	type TransferPayment = ();
	type DustRemoval = ();
}

impl<T: Trait<I>, I: Instance> Currency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDebug
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, I>;
	type NegativeImbalance = NegativeImbalance<T, I>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Self::free_balance(who) + Self::reserved_balance(who)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		Self::free_balance(who) >= value
	}

	fn total_issuance() -> Self::Balance {
		<TotalIssuance<T, I>>::get()
	}

	fn minimum_balance() -> Self::Balance {
		Self::existential_deposit()
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		<FreeBalance<T, I>>::get(who)
	}

	// # <weight>
	// Despite iterating over a list of locks, they are limited by the number of
	// lock IDs, which means the number of runtime modules that intend to use and create locks.
	// # </weight>
	fn ensure_can_withdraw(
		who: &T::AccountId,
		_amount: T::Balance,
		reason: WithdrawReason,
		new_balance: T::Balance,
	) -> Result {
		match reason {
			WithdrawReason::Reserve | WithdrawReason::Transfer if Self::vesting_balance(who) > new_balance =>
				return Err("vesting balance too high to send value"),
			_ => {}
		}
		let locks = Self::locks(who);
		if locks.is_empty() {
			return Ok(())
		}

		let now = <timestamp::Module<T>>::now();
		if locks.into_iter()
			.all(|l|
				now >= l.until
				|| new_balance >= l.amount
				|| !l.reasons.contains(reason)
			)
		{
			Ok(())
		} else {
			Err("account liquidity restrictions prevent withdrawal")
		}
	}

	fn transfer(transactor: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> Result {
		let from_balance = Self::free_balance(transactor);
		let to_balance = Self::free_balance(dest);
		let would_create = to_balance.is_zero();
		let fee = if would_create { Self::creation_fee() } else { Self::transfer_fee() };
		let liability = match value.checked_add(&fee) {
			Some(l) => l,
			None => return Err("got overflow after adding a fee to value"),
		};

		let new_from_balance = match from_balance.checked_sub(&liability) {
			None => return Err("balance too low to send value"),
			Some(b) => b,
		};
		if would_create && value < Self::existential_deposit() {
			return Err("value too low to create account");
		}
		Self::ensure_can_withdraw(transactor, value, WithdrawReason::Transfer, new_from_balance)?;

		// NOTE: total stake being stored in the same type means that this could never overflow
		// but better to be safe than sorry.
		let new_to_balance = match to_balance.checked_add(&value) {
			Some(b) => b,
			None => return Err("destination balance too high to receive value"),
		};

		if transactor != dest {
			Self::set_free_balance(transactor, new_from_balance);
			if !<FreeBalance<T, I>>::exists(dest) {
				Self::new_account(dest, new_to_balance);
			}
			Self::set_free_balance(dest, new_to_balance);
			T::TransferPayment::on_unbalanced(NegativeImbalance::new(fee));
			Self::deposit_event(RawEvent::Transfer(transactor.clone(), dest.clone(), value, fee));
		}

		Ok(())
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reason: WithdrawReason,
		liveness: ExistenceRequirement,
	) -> result::Result<Self::NegativeImbalance, &'static str> {
		if let Some(new_balance) = Self::free_balance(who).checked_sub(&value) {
			if liveness == ExistenceRequirement::KeepAlive && new_balance < Self::existential_deposit() {
				return Err("payment would kill account")
			}
			Self::ensure_can_withdraw(who, value, reason, new_balance)?;
			Self::set_free_balance(who, new_balance);
			Ok(NegativeImbalance::new(value))
		} else {
			Err("too few free funds in account")
		}
	}

	fn slash(
		who: &T::AccountId,
		value: Self::Balance
	) -> (Self::NegativeImbalance, Self::Balance) {
		let free_balance = Self::free_balance(who);
		let free_slash = cmp::min(free_balance, value);
		Self::set_free_balance(who, free_balance - free_slash);
		let remaining_slash = value - free_slash;
		// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
		// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
		// to draw from reserved funds, however we err on the side of punishment if things are inconsistent
		// or `can_slash` wasn't used appropriately.
		if !remaining_slash.is_zero() {
			let reserved_balance = Self::reserved_balance(who);
			let reserved_slash = cmp::min(reserved_balance, remaining_slash);
			Self::set_reserved_balance(who, reserved_balance - reserved_slash);
			(NegativeImbalance::new(free_slash + reserved_slash), remaining_slash - reserved_slash)
		} else {
			(NegativeImbalance::new(value), Zero::zero())
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance
	) -> result::Result<Self::PositiveImbalance, &'static str> {
		if Self::total_balance(who).is_zero() {
			return Err("beneficiary account must pre-exist");
		}
		Self::set_free_balance(who, Self::free_balance(who) + value);
		Ok(PositiveImbalance::new(value))
	}

	fn deposit_creating(
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::PositiveImbalance {
		let (imbalance, _) = Self::make_free_balance_be(who, Self::free_balance(who) + value);
		if let SignedImbalance::Positive(p) = imbalance {
			p
		} else {
			// Impossible, but be defensive.
			Self::PositiveImbalance::zero()
		}
	}

	fn make_free_balance_be(who: &T::AccountId, balance: T::Balance) -> (
		SignedImbalance<Self::Balance, Self::PositiveImbalance>,
		UpdateBalanceOutcome
	) {
		let original = Self::free_balance(who);
		if balance < Self::existential_deposit() && original.is_zero() {
			// If we're attempting to set an existing account to less than ED, then
			// bypass the entire operation. It's a no-op if you follow it through, but
			// since this is an instance where we might account for a negative imbalance
			// (in the dust cleaner of set_free_balance) before we account for its actual
			// equal and opposite cause (returned as an Imbalance), then in the
			// instance that there's no other accounts on the system at all, we might
			// underflow the issuance and our arithmetic will be off.
			return (
				SignedImbalance::Positive(Self::PositiveImbalance::zero()),
				UpdateBalanceOutcome::AccountKilled,
			)
		}
		let imbalance = if original <= balance {
			SignedImbalance::Positive(PositiveImbalance::new(balance - original))
		} else {
			SignedImbalance::Negative(NegativeImbalance::new(original - balance))
		};
		// If the balance is too low, then the account is reaped.
		// NOTE: There are two ring for every account: `reserved_balance` and
		// `free_balance`. This contract subsystem only cares about the latter: whenever
		// the term "balance" is used *here* it should be assumed to mean "free balance"
		// in the rest of the module.
		// Free balance can never be less than ED. If that happens, it gets reduced to zero
		// and the account information relevant to this subsystem is deleted (i.e. the
		// account is reaped).
		let outcome = if balance < <Module<T, I>>::existential_deposit() {
			Self::set_free_balance(who, balance);
			UpdateBalanceOutcome::AccountKilled
		} else {
			if !<FreeBalance<T, I>>::exists(who) {
				Self::new_account(&who, balance);
			}
			Self::set_free_balance(who, balance);
			UpdateBalanceOutcome::Updated
		};
		(imbalance, outcome)
	}
}

impl<T: Trait<I>, I: Instance> ReservableCurrency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDebug
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Self::free_balance(who)
			.checked_sub(&value)
			.map_or(false, |new_balance|
				Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve, new_balance).is_ok()
			)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		<ReservedBalance<T, I>>::get(who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> result::Result<(), &'static str> {
		let b = Self::free_balance(who);
		if b < value {
			return Err("not enough free funds")
		}
		let new_balance = b - value;
		Self::ensure_can_withdraw(who, value, WithdrawReason::Reserve, new_balance)?;
		Self::set_reserved_balance(who, Self::reserved_balance(who) + value);
		Self::set_free_balance(who, new_balance);
		Ok(())
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		let b = Self::reserved_balance(who);
		let actual = cmp::min(b, value);
		Self::set_free_balance(who, Self::free_balance(who) + actual);
		Self::set_reserved_balance(who, b - actual);
		value - actual
	}

	fn slash_reserved(
		who: &T::AccountId,
		value: Self::Balance
	) -> (Self::NegativeImbalance, Self::Balance) {
		let b = Self::reserved_balance(who);
		let slash = cmp::min(b, value);
		// underflow should never happen, but it if does, there's nothing to be done here.
		Self::set_reserved_balance(who, b - slash);
		(NegativeImbalance::new(slash), value - slash)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
	) -> result::Result<Self::Balance, &'static str> {
		if Self::total_balance(beneficiary).is_zero() {
			return Err("beneficiary account must pre-exist");
		}
		let b = Self::reserved_balance(slashed);
		let slash = cmp::min(b, value);
		Self::set_free_balance(beneficiary, Self::free_balance(beneficiary) + slash);
		Self::set_reserved_balance(slashed, b - slash);
		Ok(value - slash)
	}
}

impl<T: Trait<I>, I: Instance> LockableCurrency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDebug
{
	type Moment = T::Moment;

	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::Moment,
		reasons: WithdrawReasons,
	) {
		let now = <timestamp::Module<T>>::now();
		let mut new_lock = Some(BalanceLock { id, amount, until, reasons });
		let mut locks = Self::locks(who).into_iter().filter_map(|l|
			if l.id == id {
				new_lock.take()
			} else if l.until > now {
				Some(l)
			} else {
				None
			}).collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T, I>>::insert(who, locks);
	}

	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		until: T::Moment,
		reasons: WithdrawReasons,
	) {
		let now = <timestamp::Module<T>>::now();
		let mut new_lock = Some(BalanceLock { id, amount, until, reasons });
		let mut locks = Self::locks(who).into_iter().filter_map(|l|
			if l.id == id {
				new_lock.take().map(|nl| {
					BalanceLock {
						id: l.id,
						amount: l.amount.max(nl.amount),
						until: l.until.max(nl.until),
						reasons: l.reasons | nl.reasons,
					}
				})
			} else if l.until > now {
				Some(l)
			} else {
				None
			}).collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		<Locks<T, I>>::insert(who, locks);
	}

	fn remove_lock(
		id: LockIdentifier,
		who: &T::AccountId,
	) {
		let now = <timestamp::Module<T>>::now();
		let locks = Self::locks(who).into_iter().filter_map(|l|
			if l.until > now && l.id != id {
				Some(l)
			} else {
				None
			}).collect::<Vec<_>>();
		<Locks<T, I>>::insert(who, locks);
	}
}

impl<T: Trait<I>, I: Instance> MakePayment<T::AccountId> for Module<T, I> {
	fn make_payment(transactor: &T::AccountId, encoded_len: usize) -> Result {
		let encoded_len = T::Balance::from(encoded_len as u32);
		let transaction_fee = Self::transaction_base_fee() + Self::transaction_byte_fee() * encoded_len;
		let imbalance = Self::withdraw(
			transactor,
			transaction_fee,
			WithdrawReason::TransactionPayment,
			ExistenceRequirement::KeepAlive
		)?;
		T::TransactionPayment::on_unbalanced(imbalance);
		Ok(())
	}
}

impl<T: Trait<I>, I: Instance> IsDeadAccount<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDebug
{
	fn is_dead_account(who: &T::AccountId) -> bool {
		Self::total_balance(who).is_zero()
	}
}
