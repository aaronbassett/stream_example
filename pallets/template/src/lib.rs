#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxBooleansPerAccount: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type SingleFlipperValue<T> = StorageValue<_, ()>;

	#[pallet::type_value]
	pub fn PerAccountFlipperDefault<T: Config>() -> bool {
		true
	}

	#[pallet::storage]
	#[pallet::getter(fn per_account)]
	pub type PerAccountFlipperValue<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
		PerAccountFlipperDefault<T>,
	>;

	#[pallet::type_value]
	pub fn PerAccountMultipleBooleanDefault<T: Config>(
	) -> BoundedVec<bool, T::MaxBooleansPerAccount> {
		Default::default()
	}

	#[pallet::storage]
	#[pallet::getter(fn per_account_multi)]
	pub type PerAccountMultipleBooleanValues<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<bool, T::MaxBooleansPerAccount>,
		ValueQuery,
		PerAccountMultipleBooleanDefault<T>,
	>;

	#[pallet::type_value]
	pub fn LikesOrDislikesDefault<T: Config>() -> bool {
		false
	}

	#[pallet::storage]
	#[pallet::getter(fn likes)]
	pub type LikesOrDislikes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		ValueQuery,
		LikesOrDislikesDefault<T>,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		FlippedSingleValue {
			current_state: bool,
		},
		FlippedAccountValue {
			current_state: bool,
		},
		AccountBooleansUpdated {
			booleans: BoundedVec<bool, T::MaxBooleansPerAccount>,
		},

		UpdatedLikesOrDislikes {
			person1: T::AccountId,
			person2: T::AccountId,
			likes: bool,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn flip_single_value(origin: OriginFor<T>) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let _who = ensure_signed(origin)?;

			if SingleFlipperValue::<T>::exists() {
				SingleFlipperValue::<T>::kill();
				Self::deposit_event(Event::FlippedSingleValue { current_state: false });
			} else {
				SingleFlipperValue::<T>::put(());
				Self::deposit_event(Event::FlippedSingleValue { current_state: true });
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn flip_account_value(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			/*
			let mut account_value = Self::per_account(&who);
			account_value = !account_value;

			PerAccountFlipperValue::<T>::insert(&who, account_value);
			 */

			PerAccountFlipperValue::<T>::mutate(&who, |account_value| {
				*account_value = !*account_value;
			});

			Self::deposit_event(Event::FlippedAccountValue {
				current_state: Self::per_account(&who),
			});

			Ok(())
		}

		#[pallet::call_index(20)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_boolean_to_account(origin: OriginFor<T>, new_boolean: bool) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let mut account_booleans = PerAccountMultipleBooleanValues::<T>::get(&who);

			account_booleans
				.try_push(new_boolean)
				.map_err(|_| Error::<T>::StorageOverflow)?;

			PerAccountMultipleBooleanValues::<T>::insert(&who, account_booleans);

			Self::deposit_event(Event::AccountBooleansUpdated {
				booleans: PerAccountMultipleBooleanValues::<T>::get(&who),
			});

			Ok(())
		}

		#[pallet::call_index(30)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn flip_like_or_dislike(
			origin: OriginFor<T>,
			other_person: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			LikesOrDislikes::<T>::mutate(&who, &other_person, |current_status| {
				*current_status = !*current_status;
			});

			Self::deposit_event(Event::UpdatedLikesOrDislikes {
				person1: who.clone(),
				person2: other_person.clone(),
				likes: Self::likes(&who, &other_person),
			});

			Ok(())
		}
	}
}
