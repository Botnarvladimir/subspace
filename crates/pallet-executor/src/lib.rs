// Copyright (C) 2021 Subspace Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

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

//! Pallet Executor

#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::offchain::SubmitTransaction;
pub use pallet::*;
use sp_executor::{
    BundleEquivocationProof, FraudProof, InvalidTransactionProof, SignedExecutionReceipt,
    SignedOpaqueBundle,
};
use sp_runtime::RuntimeAppPublic;

#[frame_support::pallet]
mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_core::H256;
    use sp_executor::{
        BundleEquivocationProof, ExecutorId, FraudProof, InvalidTransactionProof,
        SignedExecutionReceipt, SignedOpaqueBundle,
    };
    use sp_runtime::traits::{CheckEqual, MaybeDisplay, MaybeMallocSizeOf, SimpleBitOps};
    use sp_std::fmt::Debug;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Secondary chain block hash type.
        type SecondaryHash: Parameter
            + Member
            + MaybeSerializeDeserialize
            + Debug
            + MaybeDisplay
            + SimpleBitOps
            + Ord
            + Default
            + Copy
            + CheckEqual
            + sp_std::hash::Hash
            + AsRef<[u8]>
            + AsMut<[u8]>
            + MaybeMallocSizeOf
            + MaxEncodedLen;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// The head number was wrong against the latest head.
        UnexpectedHeadNumber,
        /// Invalid bundle signature.
        BadBundleSignature,
        /// The signer of bundle is unexpected.
        UnexpectedBundleSigner,
        /// Invalid execution receipt signature.
        BadExecutionReceiptSignature,
        /// The signer of execution receipt is unexpected.
        UnexpectedExecutionReceiptSigner,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new candidate receipt was backed.
        ExecutionReceiptStored { receipt_hash: H256 },
        /// A transaction bundle was included.
        TransactionBundleStored { bundle_hash: H256 },
        /// A fraud proof was processed.
        FraudProofProcessed,
        /// A bundle equivocation proof was processed.
        BundleEquivocationProofProcessed,
        /// An invalid transaction proof was processed.
        InvalidTransactionProofProcessed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight((10_000, Pays::No))]
        pub fn submit_execution_receipt(
            origin: OriginFor<T>,
            execution_receipt: SignedExecutionReceipt<T::SecondaryHash>,
        ) -> DispatchResult {
            ensure_none(origin)?;

            log::debug!(
                target: "runtime::subspace::executor",
                "Submitting execution receipt: {:?}",
                execution_receipt
            );

            // TODO: track the execution receipt

            Self::deposit_event(Event::ExecutionReceiptStored {
                receipt_hash: execution_receipt.hash(),
            });

            Ok(())
        }

        #[pallet::weight((10_000, Pays::No))]
        pub fn submit_transaction_bundle(
            origin: OriginFor<T>,
            signed_opaque_bundle: SignedOpaqueBundle,
        ) -> DispatchResult {
            ensure_none(origin)?;

            log::debug!(
                target: "runtime::subspace::executor",
                "Submitting transaction bundle: {:?}",
                signed_opaque_bundle
            );

            Self::deposit_event(Event::TransactionBundleStored {
                bundle_hash: signed_opaque_bundle.hash(),
            });

            Ok(())
        }

        #[pallet::weight((10_000, Pays::No))]
        pub fn submit_fraud_proof(origin: OriginFor<T>, fraud_proof: FraudProof) -> DispatchResult {
            ensure_none(origin)?;

            log::debug!(
                target: "runtime::subspace::executor",
                "Submitting fraud proof: {:?}",
                fraud_proof
            );

            // TODO: slash the executor accordingly.

            Self::deposit_event(Event::FraudProofProcessed);

            Ok(())
        }

        #[pallet::weight((10_000, Pays::No))]
        pub fn submit_bundle_equivocation_proof(
            origin: OriginFor<T>,
            bundle_equivocation_proof: BundleEquivocationProof,
        ) -> DispatchResult {
            ensure_none(origin)?;

            log::debug!(
                target: "runtime::subspace::executor",
                "Submitting bundle equivocation proof: {:?}",
                bundle_equivocation_proof
            );

            // TODO: slash the executor accordingly.

            Self::deposit_event(Event::BundleEquivocationProofProcessed);

            Ok(())
        }

        #[pallet::weight((10_000, Pays::No))]
        pub fn submit_invalid_transaction_proof(
            origin: OriginFor<T>,
            invalid_transaction_proof: InvalidTransactionProof,
        ) -> DispatchResult {
            ensure_none(origin)?;

            log::debug!(
                target: "runtime::subspace::executor",
                "Submitting invalid transaction proof: {:?}",
                invalid_transaction_proof
            );

            // TODO: slash the executor accordingly.

            Self::deposit_event(Event::InvalidTransactionProofProcessed);

            Ok(())
        }
    }

    /// A tuple of (stable_executor_id, executor_signing_key).
    #[pallet::storage]
    #[pallet::getter(fn executor)]
    pub(super) type Executor<T: Config> = StorageValue<_, (T::AccountId, ExecutorId), OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub executor: Option<(T::AccountId, ExecutorId)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { executor: None }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Executor<T>>::put(
                self.executor
                    .clone()
                    .expect("Executor authority must be provided at genesis; qed"),
            );
        }
    }

    /// Constructs a `TransactionValidity` with pallet-executor specific defaults.
    fn unsigned_validity(prefix: &'static str, tag: impl Encode) -> TransactionValidity {
        ValidTransaction::with_tag_prefix(prefix)
            .priority(TransactionPriority::MAX)
            .and_provides(tag)
            .longevity(TransactionLongevity::MAX)
            // We need this extrinsic to be propagted to the farmer nodes.
            .propagate(true)
            .build()
    }

    #[repr(u8)]
    enum InvalidTransactionCode {
        BundleEquivicationProof = 101,
        TrasactionProof = 102,
        ExecutionReceipt = 103,
        Bundle = 104,
    }

    impl From<InvalidTransactionCode> for TransactionValidity {
        fn from(invalid_code: InvalidTransactionCode) -> Self {
            InvalidTransaction::Custom(invalid_code as u8).into()
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;
        fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
            match call {
                Call::submit_execution_receipt { .. } => Ok(()),
                Call::submit_transaction_bundle { .. } => Ok(()),
                Call::submit_fraud_proof { .. } => Ok(()),
                Call::submit_bundle_equivocation_proof { .. } => Ok(()),
                Call::submit_invalid_transaction_proof { .. } => Ok(()),
                _ => Err(InvalidTransaction::Call.into()),
            }
        }

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::submit_execution_receipt { execution_receipt } => {
                    if let Err(e) = Self::check_execution_receipt(execution_receipt) {
                        log::error!(
                            target: "runtime::subspace::executor",
                            "Invalid execution receipt: {:?}",
                            e
                        );
                        return InvalidTransactionCode::ExecutionReceipt.into();
                    }
                    unsigned_validity("SubspaceSubmitExecutionReceipt", execution_receipt.hash())
                }
                Call::submit_transaction_bundle {
                    signed_opaque_bundle,
                } => {
                    if let Err(e) = Self::check_bundle(signed_opaque_bundle) {
                        log::error!(
                            target: "runtime::subspace::executor",
                            "Invalid signed opaque bundle: {:?}",
                            e
                        );
                        return InvalidTransactionCode::Bundle.into();
                    }
                    unsigned_validity(
                        "SubspaceSubmitTransactionBundle",
                        signed_opaque_bundle.hash(),
                    )
                }
                Call::submit_fraud_proof { fraud_proof } => {
                    // TODO: prevent the spamming of fraud proof transaction.
                    // TODO: verify the fraud proof on the client side.
                    // if !sp_executor::fraud_proof_ext::fraud_proof::verify(fraud_proof) {
                    // log::error!(target: "runtime::subspace::executor", "Invalid fraud proof: {:?}", fraud_proof);
                    // return InvalidTransaction::Custom(INVALID_FRAUD_PROOF).into();
                    // }
                    // TODO: proper tag value.
                    unsigned_validity("SubspaceSubmitFraudProof", fraud_proof)
                }
                Call::submit_bundle_equivocation_proof {
                    bundle_equivocation_proof,
                } => {
                    if let Err(e) = Self::check_bundle_equivocation_proof(bundle_equivocation_proof)
                    {
                        log::error!(
                            target: "runtime::subspace::executor",
                            "Invalid bundle equivocation proof: {:?}",
                            e
                        );
                        return InvalidTransactionCode::BundleEquivicationProof.into();
                    }

                    unsigned_validity(
                        "SubspaceSubmitBundleEquivocationProof",
                        bundle_equivocation_proof.hash(),
                    )
                }
                Call::submit_invalid_transaction_proof {
                    invalid_transaction_proof,
                } => {
                    if let Err(e) = Self::check_invalid_transaction_proof(invalid_transaction_proof)
                    {
                        log::error!(
                            target: "runtime::subspace::executor",
                            "Wrong InvalidTransactionProof : {:?}",
                            e
                        );
                        return InvalidTransactionCode::TrasactionProof.into();
                    }

                    unsigned_validity(
                        "SubspaceSubmitInvalidTransactionProof",
                        invalid_transaction_proof,
                    )
                }

                _ => InvalidTransaction::Call.into(),
            }
        }
    }
}

impl<T: Config> Pallet<T> {
    fn check_execution_receipt(
        SignedExecutionReceipt {
            execution_receipt,
            signature,
            signer,
        }: &SignedExecutionReceipt<T::SecondaryHash>,
    ) -> Result<(), Error<T>> {
        if !signer.verify(&execution_receipt.hash(), signature) {
            return Err(Error::<T>::BadExecutionReceiptSignature);
        }

        // TODO: upgrade once the trusted executor system is upgraded.
        let expected_executor = Self::executor()
            .map(|(_, authority_id)| authority_id)
            .expect("Executor must be initialized before launching the executor chain; qed");
        if *signer != expected_executor {
            return Err(Error::<T>::UnexpectedExecutionReceiptSigner);
        }

        Ok(())
    }

    fn check_bundle(
        SignedOpaqueBundle {
            opaque_bundle,
            signature,
            signer,
        }: &SignedOpaqueBundle,
    ) -> Result<(), Error<T>> {
        if !signer.verify(&opaque_bundle.hash(), signature) {
            return Err(Error::<T>::BadBundleSignature);
        }

        // TODO: upgrade once the trusted executor system is upgraded.
        let expected_executor = Self::executor()
            .map(|(_, authority_id)| authority_id)
            .expect("Executor must be initialized before launching the executor chain; qed");
        if *signer != expected_executor {
            return Err(Error::<T>::UnexpectedBundleSigner);
        }

        Ok(())
    }

    // TODO: Checks if the bundle equivocation proof is valid.
    fn check_bundle_equivocation_proof(
        _bundle_equivocation_proof: &BundleEquivocationProof,
    ) -> Result<(), Error<T>> {
        Ok(())
    }

    // TODO: Checks if the invalid transaction proof is valid.
    fn check_invalid_transaction_proof(
        _invalid_transaction_proof: &InvalidTransactionProof,
    ) -> Result<(), Error<T>> {
        Ok(())
    }
}

impl<T> Pallet<T>
where
    T: Config + frame_system::offchain::SendTransactionTypes<Call<T>>,
{
    /// Submits an unsigned extrinsic [`Call::submit_execution_receipt`].
    pub fn submit_execution_receipt_unsigned(
        execution_receipt: SignedExecutionReceipt<T::SecondaryHash>,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let call = Call::submit_execution_receipt { execution_receipt };

        match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
            Ok(()) => {
                log::info!(target: "runtime::subspace::executor", "Submitted execution receipt.")
            }
            Err(e) => log::error!(
                target: "runtime::subspace::executor",
                "Error submitting execution receipt: {:?}",
                e
            ),
        }

        Ok(())
    }

    /// Submits an unsigned extrinsic [`Call::submit_transaction_bundle`].
    pub fn submit_transaction_bundle_unsigned(
        signed_opaque_bundle: SignedOpaqueBundle,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let call = Call::submit_transaction_bundle {
            signed_opaque_bundle,
        };

        match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
            Ok(()) => {
                log::info!(target: "runtime::subspace::executor", "Submitted transaction bundle.")
            }
            Err(e) => log::error!(
                target: "runtime::subspace::executor",
                "Error submitting transaction bundle: {:?}",
                e,
            ),
        }

        Ok(())
    }

    /// Submits an unsigned extrinsic [`Call::submit_fraud_proof`].
    pub fn submit_fraud_proof_unsigned(
        fraud_proof: FraudProof,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let call = Call::submit_fraud_proof { fraud_proof };

        match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
            Ok(()) => log::info!(target: "runtime::subspace::executor", "Submitted fraud proof."),
            Err(e) => {
                log::error!(target: "runtime::subspace::executor", "Error submitting fraud proof: {:?}", e,)
            }
        }

        Ok(())
    }

    /// Submits an unsigned extrinsic [`Call::submit_bundle_equivocation_proof`].
    pub fn submit_bundle_equivocation_proof_unsigned(
        bundle_equivocation_proof: BundleEquivocationProof,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let call = Call::submit_bundle_equivocation_proof {
            bundle_equivocation_proof,
        };

        match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
            Ok(()) => {
                log::info!(target: "runtime::subspace::executor", "Submitted bundle equivocation proof.")
            }
            Err(e) => log::error!(
                target: "runtime::subspace::executor",
                "Error submitting bundle equivocation proof: {:?}",
                e,
            ),
        }

        Ok(())
    }

    /// Submits an unsigned extrinsic [`Call::submit_invalid_transaction_proof`].
    pub fn submit_invalid_transaction_proof_unsigned(
        invalid_transaction_proof: InvalidTransactionProof,
    ) -> frame_support::pallet_prelude::DispatchResult {
        let call = Call::submit_invalid_transaction_proof {
            invalid_transaction_proof,
        };

        match SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()) {
            Ok(()) => {
                log::info!(target: "runtime::subspace::executor", "Submitted invalid transaction proof.")
            }
            Err(e) => log::error!(
                target: "runtime::subspace::executor",
                "Error submitting invalid transaction proof: {:?}",
                e,
            ),
        }

        Ok(())
    }
}
