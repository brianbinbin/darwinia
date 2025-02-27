// this crate is only for practice and verify syntax and functions.

//DONE: test initial value in Store
//DONE: test Option value in Store
//TODO: test difference between dispatch::Result & rstd::result::Result
//TODO: test visibility of functions in `decl_module!` and `impl Module` block
//TODO: check the priority between configs set in chain_spec or in module
//TODO: try out add_extra_genesis

#![cfg_attr(not(feature = "std"), no_std)]

use rstd::{cmp, result};
use rstd::prelude::*;
use support::{decl_event, decl_module, decl_storage, StorageMap, StorageValue};
use support::dispatch::Result;
use system::ensure_signed;

mod tests;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
	    SomethingStored(u32, AccountId),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		SomeOption get(someoption) config(): Option<u32>;
		Something get(something): u32;
		MapOption get(map_option): map u32 => Option<T::AccountId>;
		Map get(map): map u32 => T::AccountId;
		List get(list): map u32 => Vec<u32>;
	}
}

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		pub fn do_something(origin, something: u32) -> Result {
			let who = ensure_signed(origin)?;

			<Something<T>>::put(something);
			<SomeOption<T>>::put(something);

			// here we are raising the Something event
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		pub fn do_map(origin, uint: u32) -> Result {
		    let who = ensure_signed(origin)?;

		    <MapOption<T>>::insert(uint, who.clone());
		    <Map<T>>::insert(uint, who.clone());

            Ok(())
		}

		fn update_list(value: u32, is_add: bool) {
		    let mut list = Self::list(1);
		    if is_add {
		        list.push(value);
		        <List<T>>::insert(1, list);
		    } else {
		        list.remove(value as usize);
		    }

		}
	}
}


