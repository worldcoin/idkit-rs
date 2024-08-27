use ring::{
	aead::{self, LessSafeKey, Nonce, UnboundKey},
	rand::{SecureRandom, SystemRandom},
};
use serde_json::json;
use types::BridgeProof;
use url::Url;
use uuid::Uuid;

mod types;

use crate::{
	hashing::{base64_decode, base64_encode, encode_signal},
	Proof,
};
pub use types::{AppError, AppId, BridgeUrl, CredentialType, VerificationLevel};

/// The status of a verification request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
	/// Waiting for the World App to retrieve the request
	WaitingForConnection,
	/// Waiting for the user to confirm the request
	AwaitingConfirmation,
	/// The user has confirmed the request. Contains the proof of verification.
	Confirmed(Proof),
	/// The request has failed. Contains details about the failure.
	Failed(AppError),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Payload {
	iv: String,
	payload: String,
}

#[derive(Debug, serde::Deserialize)]
struct BridgeCreateResponse {
	request_id: Uuid,
}

#[derive(Debug, serde::Deserialize)]
struct BridgePollResponse {
	status: String,
	response: Option<Payload>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum BridgeResponse {
	Error { error_code: AppError },
	Success(BridgeProof),
}

/// A session with the Wallet Bridge.
#[derive(Debug)]
pub struct Session {
	key: LessSafeKey,
	request_id: Uuid,
	key_bytes: Vec<u8>,
	bridge_url: BridgeUrl,
	client: reqwest::Client,
}

/// An error when interacting with the Wallet Bridge.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("An error occurred when communicating with the Wallet Bridge: {0}")]
	Bridge(#[from] reqwest::Error),

	#[error("An error occurred when encoding or decoding a request or response: {0}")]
	Json(#[from] serde_json::Error),

	#[error("An error occurred when generating a key, encrypting or decrypting a request or response: {0}")]
	Encryption(&'static str),

	#[error("An error occurred when base64 encoding or decoding a request or response: {0}")]
	Base64(#[from] base64::DecodeError),
}

impl Session {
	/// Create a new session with the Wallet Bridge.
	///
	/// # Errors
	///
	/// Returns an error if the request to the bridge fails, or if the response from the bridge is malformed.
	pub async fn new<V: alloy_sol_types::SolValue + Send>(
		app_id: &AppId,
		action: &str,
		verification_level: VerificationLevel,
		bridge_url: BridgeUrl,
		signal: V,
		action_description: Option<&str>,
	) -> Result<Self, Error> {
		let client = reqwest::Client::builder()
			.user_agent(format!(
				"{}/{}",
				env!("CARGO_PKG_NAME"),
				env!("CARGO_PKG_VERSION")
			))
			.build()?;

		let (key_bytes, key, iv) = Self::generate_key()?;

		let response = client
			.post(
				bridge_url
					.join("/request")
					.unwrap_or_else(|_| unreachable!()),
			)
			.json(&Self::encrypt_request(
				&key,
				iv,
				&json!({
					"app_id": app_id,
					"action": action,
					"action_description": action_description,
					"signal": format!("0x{:x}", encode_signal(&signal)),
					"verification_level": verification_level.to_string(),
					"credential_types": verification_level.to_credential_types(),
				}),
			)?)
			.send()
			.await?
			.json::<BridgeCreateResponse>()
			.await?;

		Ok(Self {
			key,
			client,
			key_bytes,
			bridge_url,
			request_id: response.request_id,
		})
	}

	/// Returns the URL that the user should be directed to in order to connect their World App to the client.
	#[must_use]
	pub fn connect_url(&self) -> Url {
		Url::parse(&format!(
			"https://worldcoin.org/verify?t=wld&i={}&k={}{}",
			self.request_id,
			urlencoding::encode(&base64_encode(&self.key_bytes)),
			if self.bridge_url == BridgeUrl::default() {
				String::new()
			} else {
				format!("&b={}", &self.bridge_url.0)
			}
		))
		.unwrap_or_else(|_| unreachable!())
	}

	/// Polls the bridge for the status of the request, and returns the current status.
	/// You should call this method repeatedly until it returns `Status::Confirmed` or `Status::Failed`. Calling it again after leads to undefined behaviour.
	///
	/// # Errors
	///
	/// Returns an error if the request to the bridge fails, or if the response from the bridge is malformed.
	pub async fn poll_for_status(&self) -> Result<Status, Error> {
		let response = self
			.client
			.get(
				self.bridge_url
					.join(&format!("/response/{}", self.request_id))
					.unwrap_or_else(|_| unreachable!()),
			)
			.send()
			.await?;

		if !response.status().is_success() {
			return Ok(Status::Failed(AppError::ConnectionFailed));
		}

		let response = response.json::<BridgePollResponse>().await?;

		if response.status != "completed" {
			return Ok(match response.status.as_str() {
				"retrieved" => Status::AwaitingConfirmation,
				"initialized" => Status::WaitingForConnection,
				_ => unreachable!("Invalid status returned from bridge"),
			});
		}

		match self.decrypt_response(&response.response.unwrap_or_else(|| unreachable!()))? {
			BridgeResponse::Error { error_code } => Ok(Status::Failed(error_code)),
			BridgeResponse::Success(proof) => Ok(Status::Confirmed(proof.into())),
		}
	}

	fn generate_key() -> Result<(Vec<u8>, LessSafeKey, Nonce), Error> {
		let rand = SystemRandom::new();

		let mut iv = [0; aead::NONCE_LEN];
		rand.fill(&mut iv[..])
			.map_err(|_| Error::Encryption("Failed to generate IV"))?;

		let mut key_bytes: [u8; 32] = [0; 32];
		rand.fill(&mut key_bytes)
			.map_err(|_| Error::Encryption("Failed to generate key"))?;

		let key = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
			.map_err(|_| Error::Encryption("AES-256-GCM is a supported algorithm"))?;

		Ok((
			key_bytes.to_vec(),
			LessSafeKey::new(key),
			Nonce::assume_unique_for_key(iv),
		))
	}

	fn encrypt_request(
		key: &LessSafeKey,
		nonce: Nonce,
		payload: &serde_json::Value,
	) -> Result<Payload, Error> {
		let iv = base64_encode(nonce.as_ref());
		let mut payload = serde_json::to_vec(&payload)?;

		key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut payload)
			.map_err(|_| Error::Encryption("Failed to encrypt bridge request"))?;

		Ok(Payload {
			iv,
			payload: base64_encode(payload),
		})
	}

	fn decrypt_response(&self, payload: &Payload) -> Result<BridgeResponse, Error> {
		let nonce = Nonce::try_assume_unique_for_key(&base64_decode(&payload.iv)?)
			.map_err(|_| Error::Encryption("Invalid IV"))?;

		let mut payload = base64_decode(&payload.payload)?;
		let payload = self
			.key
			.open_in_place(nonce, aead::Aad::empty(), &mut payload)
			.map_err(|_| Error::Encryption("Failed to decrypt bridge response"))?;

		Ok(serde_json::from_slice(payload)?)
	}
}
