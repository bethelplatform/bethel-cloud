#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::Vec,
		pallet_prelude::*,
		traits::{Currency, Randomness},
		Twox64Concat,
	};
	use frame_system::pallet_prelude::*;
	#[pallet::pallet]
	#[pallet::without_storage_info]
	// #[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
		// type CloudIdRandomness: Randomness<H256, u32>;

		#[pallet::constant]
		type MaximumOwned: Get<u32>;
	}

	// type BalanceOf<T> =
	// 	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CloudID<T: Config> {
		// Unsigned integers of 16 bytes to represent a unique identifier
		pub unique_id: [u8; 16],
		pub path: Vec<u8>,
		pub owner: T::AccountId,
	}

	#[pallet::storage]
	pub(super) type CloudIDCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type OwnerOfCloudID<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<[u8; 16], T::MaximumOwned>,
		ValueQuery,
	>;

	/// Maps the Collectible struct to the unique_id.
	#[pallet::storage]
	pub(super) type CloudIdMap<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], CloudID<T>>;

	#[pallet::error]
	pub enum Error<T> {
		/// Each CoudID must have a unique identifier
		DuplicateCloudId,
		/// An account can't exceed the `MaximumOwned` constant
		MaximumCloudIdsOwned,
		/// The total supply of CloudId can't exceed the u64 limit
		BoundsOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new CloudId was successfully created
		CloudIdCreated { cloudid: [u8; 16], owner: T::AccountId },
	}

	// #[pallet::call]
	impl<T: Config> Pallet<T> {
		// pub fn gen_unique_id() -> [u8; 16] {
		// 	let random = T::CloudIdRandomness::random(&b"unique_id"[..]).0;
		// 	let unique_payload = (
		// 		random,
		// 		frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
		// 		frame_system::Pallet::<T>::block_number(),
		// 	);

		// 	let encoded_payload = unique_payload.encode();
		// 	let hash = frame_support::Hashable::blake2_128(&encoded_payload);
		// 	hash
		// }

		fn mint(
			owner: &T::AccountId,
			unique_id: [u8; 16],
			path: Vec<u8>,
		) -> Result<[u8; 16], DispatchError> {
			let cloud_id = CloudID::<T> { unique_id, path, owner: owner.clone() };
			ensure!(
				!CloudIdMap::<T>::contains_key(&cloud_id.unique_id),
				Error::<T>::DuplicateCloudId
			);

			let count = CloudIDCount::<T>::get();
			let new_count = count.checked_add(1).ok_or(Error::<T>::BoundsOverflow)?;

			OwnerOfCloudID::<T>::try_append(&owner, cloud_id.unique_id)
				.map_err(|_| Error::<T>::MaximumCloudIdsOwned)?;

			CloudIdMap::<T>::insert(cloud_id.unique_id, cloud_id);
			CloudIDCount::<T>::put(new_count);

			Self::deposit_event(Event::CloudIdCreated { cloudid: unique_id, owner: owner.clone() });

			Ok(unique_id)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_cloud_id(
			origin: OriginFor<T>,
			unique_id: [u8; 16],
			path: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// let u_id = Self::gen_unique_id();

			let _ = Self::mint(&sender, unique_id, path);

			Ok(())
		}
	}
}
