use base64::{DecodeError, Engine};
use ruint::aliases::U256;
use tiny_keccak::{Hasher, Keccak};

/// Hashes an input using the `keccak256` hashing function used across the World ID protocol, to be used as a ZKP input.
#[must_use]
pub fn hash_to_field(input: &[u8]) -> U256 {
	let n = U256::try_from_be_slice(&keccak256(input))
		.unwrap_or_else(|| unreachable!("target uint is large enough"));

	// Shift right one byte to make it fit in the field
	n >> 8
}

pub(crate) fn encode_signal<V: alloy_sol_types::SolValue>(signal: &V) -> U256 {
	hash_to_field(&signal.abi_encode_packed())
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
	let mut output = [0; 32];

	let mut hasher = Keccak::v256();
	hasher.update(bytes);
	hasher.finalize(&mut output);

	output
}

pub(crate) fn base64_encode<T: AsRef<[u8]>>(input: T) -> String {
	base64::engine::general_purpose::STANDARD.encode(input)
}
pub(crate) fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
	base64::engine::general_purpose::STANDARD.decode(input)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_hash_to_field() {
		assert_eq!(
			hash_to_field(b"hello world"),
			U256::from_str_radix(
				"125606243838566630058575099447702412745558900339761109861010052356172984351",
				10,
			)
			.unwrap()
		);
		assert_eq!(
			format!("0x{:x}", hash_to_field(b"test")),
			"0x009c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb6"
		);
	}

	#[test]
	fn test_encode_signal() {
		assert_eq!(
			format!("0x{:x}", encode_signal(&"test")),
			"0x009c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb6"
		);
		assert_eq!(
			format!("0x{:x}", encode_signal(&(U256::from(1), "test"))),
			"0x0088c8c90482320f18b0c0842feaeab88065fd7ef3ef7b06066af823d8eef6f9"
		);
		assert_eq!(
			format!("0x{:x}", encode_signal::<()>(&())),
			"0x00c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4"
		);
	}
}
