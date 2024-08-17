use std::collections::BTreeSet;

use did_simple::crypto::ed25519;
use jose_jwk::Jwk;

/// Creates a JWK from a ed25519 verifying key.
pub fn ed25519_pub_jwk(pub_key: ed25519::VerifyingKey) -> Jwk {
	Jwk {
		key: jose_jwk::Okp {
			crv: jose_jwk::OkpCurves::Ed25519,
			x: pub_key.into_inner().as_bytes().as_slice().to_owned().into(),
			d: None,
		}
		.into(),
		prm: jose_jwk::Parameters {
			ops: Some(BTreeSet::from([jose_jwk::Operations::Verify])),
			..Default::default()
		},
	}
}

#[cfg(test)]
mod test {
	use base64::Engine as _;

	use super::*;

	#[test]
	fn pub_jwk_test_vectors() {
		// See https://datatracker.ietf.org/doc/html/rfc8037#appendix-A.2
		let rfc_example = serde_json::json! ({
			"kty": "OKP",
			"crv": "Ed25519",
			"x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"
		});
		let pubkey_bytes = hex_literal::hex!(
			"d7 5a 98 01 82 b1 0a b7 d5 4b fe d3 c9 64 07 3a
            0e e1 72 f3 da a6 23 25 af 02 1a 68 f7 07 51 1a"
		);
		assert_eq!(
			base64::prelude::BASE64_URL_SAFE_NO_PAD
				.decode(rfc_example["x"].as_str().unwrap())
				.unwrap(),
			pubkey_bytes,
			"sanity check: example bytes should match, they come from the RFC itself"
		);

		let input_key = ed25519::VerifyingKey::try_from_bytes(&pubkey_bytes).unwrap();
		let mut output_jwk = ed25519_pub_jwk(input_key);

		// Check all additional outputs for expected values
		assert_eq!(
			output_jwk.prm.ops.take().unwrap(),
			BTreeSet::from([jose_jwk::Operations::Verify]),
			"expected Verify as a supported operation"
		);
		let output_jwk = output_jwk; // Freeze mutation from here on out

		// Check serialization and deserialization against the rfc example
		assert_eq!(
			serde_json::from_value::<Jwk>(rfc_example.clone()).unwrap(),
			output_jwk,
			"deserializing json to Jwk did not match"
		);
		assert_eq!(
			rfc_example,
			serde_json::to_value(output_jwk).unwrap(),
			"serializing Jwk to json did not match"
		);
	}
}
