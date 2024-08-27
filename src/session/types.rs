use std::{fmt::Display, ops::Deref, str::FromStr};
use url::Url;

use crate::Proof;

const DEFAULT_BRIDGE_URL: &str = "https://bridge.worldcoin.org";

/// The strongest credential with which a user has been verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialType {
	Orb,
	Device,
}

impl From<CredentialType> for VerificationLevel {
	fn from(val: CredentialType) -> Self {
		match val {
			CredentialType::Orb => Self::Orb,
			CredentialType::Device => Self::Device,
		}
	}
}

/// The minimum verification level accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationLevel {
	Orb,
	Device,
}

impl Default for VerificationLevel {
	fn default() -> Self {
		Self::Orb
	}
}

impl Display for VerificationLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Orb => write!(f, "orb"),
			Self::Device => write!(f, "device"),
		}
	}
}

impl VerificationLevel {
	#[must_use]
	pub fn to_credential_types(&self) -> Vec<CredentialType> {
		match self {
			Self::Orb => vec![CredentialType::Orb],
			Self::Device => vec![CredentialType::Orb, CredentialType::Device],
		}
	}
}

/// The error returned by the World App.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum AppError {
	/// Failed to connect to the World App. Please create a new session and try again.
	#[error("Failed to connect to the World App. Please create a new session and try again.")]
	ConnectionFailed,
	/// The user rejected the verification request in the World App.
	#[error("The user rejected the verification request in the World App.")]
	VerificationRejected,
	/// The user already verified the maximum number of times for this action.
	#[error("The user already verified the maximum number of times for this action.")]
	MaxVerificationsReached,
	/// The user does not have the verification level required by this app.
	#[error("The user does not have the verification level required by this app.")]
	CredentialUnavailable,
	/// There was a problem with this request. Please try again or contact the app owner.
	#[error("There was a problem with this request. Please try again or contact the app owner.")]
	MalformedRequest,
	/// Invalid network. If you are the app owner, visit docs.worldcoin.org/test for details.
	#[error(
		"Invalid network. If you are the app owner, visit docs.worldcoin.org/test for details."
	)]
	InvalidNetwork,
	/// There was an issue fetching the user's credential. Please try again.
	#[error("There was an issue fetching the user's credential. Please try again.")]
	InclusionProofFailed,
	/// The user's identity is still being registered. Please wait a few minutes and try again.
	#[error(
		"The user's identity is still being registered. Please wait a few minutes and try again."
	)]
	InclusionProofPending,
	/// Unexpected response from the user's World App. Please try again.
	#[error("Unexpected response from the user's World App. Please try again.")]
	UnexpectedResponse,
	/// Verification failed by the app. Please contact the app owner for details.
	#[error("Verification failed by the app. Please contact the app owner for details.")]
	FailedByHostApp,
	/// Something unexpected went wrong. Please try again.
	#[error("Something unexpected went wrong. Please try again.")]
	GenericError,
}

/// Unique identifier for the app verifying the action. This should be the App ID obtained from the [Developer Portal](https://developer.worldcoin.org).
#[repr(transparent)]
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AppId(pub(crate) String);

/// Error returned when an invalid app id is provided.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("Invalid app id provided, expected app_*, got {0}")]
pub struct AppIdError(String);

impl AppId {
	/// Whether this app id represents a staging app.
	#[must_use]
	pub fn is_staging(&self) -> bool {
		self.0.contains("staging")
	}

	/// Create a new app id from a string, bypasing the validation.
	///
	/// # Safety
	///
	/// The string must be a valid app id.
	#[must_use]
	pub const unsafe fn new_unchecked(app_id: String) -> Self {
		Self(app_id)
	}
}

impl FromStr for AppId {
	type Err = AppIdError;

	fn from_str(app_id: &str) -> Result<Self, Self::Err> {
		if app_id.starts_with("app_") {
			Ok(Self(app_id.to_string()))
		} else {
			Err(AppIdError(app_id.to_string()))
		}
	}
}

impl Deref for AppId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// The URL of the Wallet Bridge to use for establishing a connection with the user's World App. Defaults to the bridge service hosted by Worldcoin. Only change this if you are running your own bridge service.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct BridgeUrl(pub(crate) url::Url);

#[derive(Debug, thiserror::Error)]
pub enum BridgeUrlError {
	#[error("Bridge URL must use HTTPS.")]
	NotHttps,

	#[error("Bridge URL must use the default port.")]
	NotDefaultPort,

	#[error("Bridge URL must not contain a path.")]
	ContainsPath,

	#[error("Bridge URL must not contain a query.")]
	ContainsQuery,

	#[error("Bridge URL must not contain a fragment.")]
	ContainsFragment,
}

impl Default for BridgeUrl {
	fn default() -> Self {
		Self(Url::parse(DEFAULT_BRIDGE_URL).unwrap())
	}
}

impl Deref for BridgeUrl {
	type Target = Url;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl TryFrom<Url> for BridgeUrl {
	type Error = BridgeUrlError;

	fn try_from(url: Url) -> Result<Self, Self::Error> {
		if ["localhost", "127.0.0.1"].contains(&url.host_str().unwrap()) {
			return Ok(Self(url));
		};

		if url.scheme() != "https" {
			return Err(BridgeUrlError::NotHttps);
		}

		if url.port().is_some() {
			return Err(BridgeUrlError::NotDefaultPort);
		}

		if url.path() != "/" {
			return Err(BridgeUrlError::ContainsPath);
		}

		if url.query().is_some() {
			return Err(BridgeUrlError::ContainsQuery);
		}

		if url.fragment().is_some() {
			return Err(BridgeUrlError::ContainsFragment);
		}

		Ok(Self(url))
	}
}

/// The proof of verification returned by the World ID Bridge.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BridgeProof {
	/// The Zero-knowledge proof of the verification. A hex string, ABI encoded.
	pub proof: String,
	/// The hash pointer to the root of the Merkle tree that proves membership of the user's identity in the list of identities verified by the Orb. A hex string, ABI encoded.
	pub merkle_root: String,
	/// Essentially the user's unique identifier for your app (and specific action if using Incognito Actions). A hex string, ABI encoded.
	pub nullifier_hash: String,
	/// Either orb or device.
	pub credential_type: CredentialType,
}

impl From<BridgeProof> for Proof {
	fn from(val: BridgeProof) -> Self {
		Self {
			proof: val.proof,
			merkle_root: val.merkle_root,
			nullifier_hash: val.nullifier_hash,
			verification_level: val.credential_type.into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_app_id() {
		assert_eq!(AppId::from_str("app_123").unwrap().0, "app_123");
		assert_eq!(
			AppId::from_str("test").unwrap_err(),
			AppIdError("test".to_string())
		);

		assert!(!AppId::from_str("app_123").unwrap().is_staging());
		assert!(AppId::from_str("app_staging_123").unwrap().is_staging());
	}
}
