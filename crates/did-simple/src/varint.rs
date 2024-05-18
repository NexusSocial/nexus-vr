pub const fn decode_varint(encoded: &[u8]) -> Result<u16, DecodeError> {
	// TODO: Technically, some three byte encodings could fit into a u16, we
	// should support those in the future.
	// Luckily none of them are used for did:key afaik.
	if encoded.len() > 2 {
		return Err(DecodeError::WouldOverflow);
	}
	if encoded.is_empty() {
		return Err(DecodeError::MissingBytes);
	}

	/// bitmask for 7 least significant bits
	const LSB_7: u8 = u8::MAX / 2;
	/// bitmask for most significant bit
	const MSB: u8 = !LSB_7;

	#[inline]
	const fn msb_is_1(val: u8) -> bool {
		val & MSB == MSB
	}

	let a = encoded[0];
	let mut result: u16 = (a & LSB_7) as u16;
	if msb_is_1(a) {
		// There is another 7 bits to decode.
		if encoded.len() < 2 {
			return Err(DecodeError::MissingBytes);
		}
		let b = encoded[1];

		result |= ((b & LSB_7) as u16) << 7;
		if msb_is_1(b) {
			// We were provided a varint that ought to have had at least another byte.
			return Err(DecodeError::MissingBytes);
		}
	}
	Ok(result)
}

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum DecodeError {
	#[error("expected more bytes than what were provided")]
	MissingBytes,
	#[error(
		"the decoded number is too large to fit into the type without overflowing"
	)]
	WouldOverflow,
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_known_examples() {
		// See https://github.com/multiformats/unsigned-varint/blob/16bf9f7d3ff78c10c1ab26d397c03c91205cd4ee/README.md
		let examples1 = [
			(0x01, [0x01].as_slice()),
			(0x02, &[0x02]),
			(0x7f, &[0x7f]),         // 127
			(0x80, &[0x80, 0x01]),   // 128
			(0x81, &[0x81, 0x01]),   // 129
			(0xff, &[0xff, 0x01]),   // 255
			(0x012c, &[0xac, 0x02]), // 300
		];
		let examples2 = [
			(0xed, [0xed, 0x01].as_slice()), // ed25519
			(0xec, &[0xec, 0x01]),           // x25519
			(0x1200, &[0x80, 0x24]),         // P256
			(0xe7, &[0xe7, 0x01]),           // Secp256k1
		];

		for (decoded, encoded) in examples1.into_iter().chain(examples2) {
			assert_eq!(Ok(decoded), decode_varint(encoded))
		}
	}
}
