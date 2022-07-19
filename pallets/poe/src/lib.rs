#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		(<T as frame_system::Config>::AccountId, BlockNumberFor<T>),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(<T as frame_system::Config>::AccountId, Vec<u8>),
		ClaimRemoved(<T as frame_system::Config>::AccountId, Vec<u8>),
		// 转移事件
		ClaimTransfer(
			Vec<u8>,
			<T as frame_system::Config>::AccountId,
			<T as frame_system::Config>::AccountId,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		ProffAlreadyExists,
		ClaimNoExists,
		NoClaimOwner,
		TransError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProffAlreadyExists);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			Proofs::<T>::insert(&claim, (sender.clone(), current_block_number));
			Self::deposit_event(Event::<T>::ClaimCreated(sender, claim));
			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNoExists)?;
			ensure!(sender == owner, Error::<T>::NoClaimOwner);
			Proofs::<T>::remove(&claim);
			Self::deposit_event(Event::<T>::ClaimRemoved(sender, claim));
			Ok(().into())
		}
		// 转移存证
		#[pallet::weight(0)]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			target: <T as frame_system::Config>::AccountId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNoExists)?;
			ensure!(sender == owner, Error::<T>::NoClaimOwner);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// 改变key下的值
			// Proofs::<T>::remove(&claim);
			// Proofs::<T>::insert(&claim, (target.clone(), current_block_number));

			Proofs::<T>::try_mutate(&claim, |trans_value| match trans_value {
				Some(trans) => {
					*trans = (target.clone(), current_block_number);
					Ok(())
				}
				None => Err(()),
			})
			.map_err(|_| <Error<T>>::TransError)?;
			// 发送事件
			Self::deposit_event(Event::<T>::ClaimTransfer(claim, sender, target));
			Ok(().into())
		}
	}
}
