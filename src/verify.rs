use reqwest::{header, StatusCode};
use serde::Serialize;

use crate::{
	hashing::hash_to_field,
	session::{AppId, VerificationLevel},
	Proof,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("verification failed: {0:?}")]
	Verification(ErrorResponse),
	#[error("fail to send request: {0}")]
	Reqwest(#[from] reqwest::Error),
	#[error("failed to decode response: {0}")]
	Serde(#[from] serde_json::Error),
	#[error("unexpected response")]
	InvalidResponse(reqwest::Response),
}

#[derive(Debug, serde::Deserialize)]
pub struct ErrorResponse {
	pub code: String,
	pub detail: String,
	pub attribute: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerificationRequest {
	action: String,
	proof: String,
	merkle_root: String,
	nullifier_hash: String,
	verification_level: VerificationLevel,
	#[serde(skip_serializing_if = "Option::is_none")]
	signal_hash: Option<String>,
}

/// Verify a World ID proof using the Developer Portal API.
///
/// # Errors
///
/// Errors if the proof is invalid (`Error::Verification`), or if there's an error validating the proof.
#[allow(clippy::module_name_repetitions)]
pub async fn verify_proof<V: alloy_sol_types::SolValue + Send>(
	proof: Proof,
	app_id: AppId,
	action: &str,
	signal: V,
) -> Result<(), Error> {
	let signal = signal.abi_encode_packed();

	let response = reqwest::Client::new()
		.post(format!(
			"https://developer.worldcoin.org/api/v2/verify/{}",
			app_id.0
		))
		.header(header::USER_AGENT, "idkit-rs")
		.json(&VerificationRequest {
			proof: proof.proof,
			signal_hash: if signal.is_empty() {
				None
			} else {
				Some(format!("0x{:x}", hash_to_field(&signal)))
			},
			action: action.to_string(),
			merkle_root: proof.merkle_root,
			nullifier_hash: proof.nullifier_hash,
			verification_level: proof.verification_level,
		})
		.send()
		.await?;

	match response.status() {
		StatusCode::OK => Ok(()),
		StatusCode::BAD_REQUEST => {
			Err(Error::Verification(response.json::<ErrorResponse>().await?))
		},
		_ => Err(Error::InvalidResponse(response)),
	}
}
