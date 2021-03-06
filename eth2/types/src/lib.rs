//! Ethereum 2.0 types

// Required for big type-level numbers
#![recursion_limit = "128"]

#[macro_use]
pub mod test_utils;

pub mod attestation;
pub mod attestation_data;
pub mod attestation_duty;
pub mod attester_slashing;
pub mod beacon_block;
pub mod beacon_block_body;
pub mod beacon_block_header;
pub mod beacon_committee;
pub mod beacon_state;
pub mod chain_spec;
pub mod checkpoint;
pub mod deposit;
pub mod deposit_data;
pub mod eth1_data;
pub mod eth_spec;
pub mod fork;
pub mod free_attestation;
pub mod historical_batch;
pub mod indexed_attestation;
pub mod pending_attestation;
pub mod proposer_slashing;
pub mod utils;
pub mod voluntary_exit;
#[macro_use]
pub mod slot_epoch_macros;
pub mod relative_epoch;
pub mod slot_epoch;
pub mod slot_height;
mod tree_hash_impls;
pub mod validator;

use ethereum_types::{H160, H256};

pub use crate::attestation::{Attestation, Error as AttestationError};
pub use crate::attestation_data::AttestationData;
pub use crate::attestation_duty::AttestationDuty;
pub use crate::attester_slashing::AttesterSlashing;
pub use crate::beacon_block::BeaconBlock;
pub use crate::beacon_block_body::BeaconBlockBody;
pub use crate::beacon_block_header::BeaconBlockHeader;
pub use crate::beacon_committee::{BeaconCommittee, OwnedBeaconCommittee};
pub use crate::beacon_state::{Error as BeaconStateError, *};
pub use crate::chain_spec::{ChainSpec, Domain, YamlConfig};
pub use crate::checkpoint::Checkpoint;
pub use crate::deposit::{Deposit, DEPOSIT_TREE_DEPTH};
pub use crate::deposit_data::DepositData;
pub use crate::eth1_data::Eth1Data;
pub use crate::fork::Fork;
pub use crate::free_attestation::FreeAttestation;
pub use crate::historical_batch::HistoricalBatch;
pub use crate::indexed_attestation::IndexedAttestation;
pub use crate::pending_attestation::PendingAttestation;
pub use crate::proposer_slashing::ProposerSlashing;
pub use crate::relative_epoch::{Error as RelativeEpochError, RelativeEpoch};
pub use crate::slot_epoch::{Epoch, Slot};
pub use crate::slot_height::SlotHeight;
pub use crate::validator::Validator;
pub use crate::voluntary_exit::VoluntaryExit;

pub type CommitteeIndex = u64;
pub type Hash256 = H256;
pub type Address = H160;

pub use bls::{
    AggregatePublicKey, AggregateSignature, Keypair, PublicKey, PublicKeyBytes, SecretKey,
    Signature, SignatureBytes,
};
pub use ssz_types::{typenum, typenum::Unsigned, BitList, BitVector, FixedVector, VariableList};
