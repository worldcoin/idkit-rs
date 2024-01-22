#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod hashing;
pub mod session;

use session::CredentialType;
pub use session::Session;

/// The proof of verification returned by the World ID protocol.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Proof {
	/// The Zero-knowledge proof of the verification. A hex string, ABI encoded.
	pub proof: String,
	/// The hash pointer to the root of the Merkle tree that proves membership of the user's identity in the list of identities verified by the Orb. A hex string, ABI encoded.
	pub merkle_root: String,
	/// Essentially the user's unique identifier for your app (and specific action if using Incognito Actions). A hex string, ABI encoded.
	pub nullifier_hash: String,
	/// Either orb or device. Will always return the strongest credential with which a user has been verified.
	pub credential_type: CredentialType,
}
