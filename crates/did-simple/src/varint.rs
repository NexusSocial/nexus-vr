/// bitmask for 7 least significant bits
const LSB_7: u8 = u8::MAX / 2;
/// bitmask for most significant bit
const MSB: u8 = !LSB_7;

#[inline]
const fn msb_is_1(val: u8) -> bool {
	val & MSB == MSB
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub(crate) struct VarintEncoding {
	buf: [u8; Self::MAX_LEN],
	len: u8,
}

impl VarintEncoding {
	pub const MAX_LEN: usize = 3;

	#[allow(dead_code)]
	pub const fn as_slice(&self) -> &[u8] {
		self.buf.split_at(self.len as usize).0
	}
}

/// Counts the length of the bits in `value`.
const fn bitlength(value: u16) -> u8 {
	// ilog2 can be used to get the (0-indexed from LSB position) of the MSB.
	// We then add one to it, since we are trying to indicate length, not position.
	if let Some(x) = value.checked_ilog2() {
		(x + 1) as u8
	} else {
		0
	}
}

#[cfg(test)]
mod bitlength_test {
	use super::bitlength;
	#[test]
	fn test_bitlength() {
		assert_eq!(bitlength(0b000), 0);
		assert_eq!(bitlength(0b001), 1);
		assert_eq!(bitlength(0b010), 2);
		assert_eq!(bitlength(0b011), 2);
		assert_eq!(bitlength(0b100), 3);
	}
}

/// Encodes a value as a varint.
/// Returns an array as well as the length of the array to slice., along  well as an array.
#[allow(dead_code)]
pub(crate) const fn encode_varint(value: u16) -> VarintEncoding {
	let mut out_buf = [0; VarintEncoding::MAX_LEN];
	let in_bit_length: u16 = bitlength(value) as u16;
	if in_bit_length == 0 {
		return VarintEncoding {
			buf: out_buf,
			len: 1,
		};
	};

	let mut out = 0u32;
	let mut in_bit_pos = 0u16;
	// Each time we need to write a carry bit into the MSB, we increment this.
	let mut carry_counter = 0;
	// Have to use macro because consts can't use closures or &mut in functions :(
	macro_rules! copy_chunk {
		() => {
			let retrieved_bits = value & ((LSB_7 as u16) << in_bit_pos);
			// if the carry bits weren't a thing, we could keep the position intact
			// and just directly copy the chunk. But we need to shift left to account
			// for prior carry bits
			out |= (retrieved_bits as u32) << carry_counter;
			in_bit_pos += 7;
		};
	}
	copy_chunk!();
	while in_bit_pos < in_bit_length {
		carry_counter += 1;
		out |= 1 << (carry_counter * 8 - 1); // turns on MSB to indicate carry.
		copy_chunk!();
	}

	// the number of bytes to write is equivalent to the carry counter + 1.
	let num_output_bytes = carry_counter + 1;

	// No for loops or copy_from_slice in const fn :(
	let mut idx = 0;
	let output_bytes = out.to_le_bytes();
	while idx < num_output_bytes {
		out_buf[idx] = output_bytes[idx];
		idx += 1;
	}

	VarintEncoding {
		buf: out_buf,
		len: num_output_bytes as u8,
	}
}

/// Returns tuple of decoded varint and rest of buffer
pub(crate) const fn decode_varint(encoded: &[u8]) -> Result<(u16, &[u8]), DecodeError> {
	if encoded.is_empty() {
		return Err(DecodeError::MissingBytes);
	}
	let mut decoded: u16 = 0;
	let mut current_encoded_idx = 0;
	let mut current_decoded_bit = 0;
	while current_decoded_bit < u16::BITS {
		if current_encoded_idx >= encoded.len() {
			return Err(DecodeError::MissingBytes);
		}
		let current_byte = encoded[current_encoded_idx];
		let current_byte_lower7 = current_byte & LSB_7;
		if current_decoded_bit
			> u16::BITS - (bitlength(current_byte_lower7 as u16) as u32)
		{
			return Err(DecodeError::WouldOverflow);
		}
		let shifted = (current_byte_lower7 as u16) << current_decoded_bit;
		decoded |= shifted;
		current_decoded_bit += 7;
		current_encoded_idx += 1;
		if !msb_is_1(current_byte) {
			let bytes_in_varint = current_encoded_idx;
			return Ok((decoded, encoded.split_at(bytes_in_varint).1));
		}
	}
	Err(DecodeError::WouldOverflow)
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
	use std::collections::HashMap;

	use itertools::Itertools;

	use super::*;

	fn test_roundtrip(decoded: u16) {
		let extra_bytes = [1, 2, 3, 4].as_slice();
		let empty = [].as_slice();
		let encoded = encode_varint(decoded);

		let round_tripped = decode_varint(encoded.as_slice());
		assert_eq!(Ok((decoded, empty)), round_tripped);
		let mut encoded_with_extra = encoded.as_slice().to_vec();
		encoded_with_extra.extend_from_slice(extra_bytes);
		let round_tripped_with_extra = decode_varint(&encoded_with_extra);
		assert_eq!(Ok((decoded, extra_bytes)), round_tripped_with_extra)
	}

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

		let empty: &[u8] = &[];
		let extra_bytes = [1, 2, 3, 4];
		for (decoded, encoded) in examples1.into_iter().chain(examples2) {
			assert_eq!(encoded, encode_varint(decoded).as_slice());
			assert_eq!(Ok((decoded, empty)), decode_varint(encoded));
			// Do another check with some additional extra bytes in the encoding
			let mut extended_encoded = encoded.to_vec();
			extended_encoded.extend_from_slice(&extra_bytes);
			assert_eq!(
				Ok((decoded, extra_bytes.as_slice())),
				decode_varint(&extended_encoded)
			);
			test_roundtrip(decoded);
		}
	}

	#[test]
	fn test_all_u16_roundtrip() {
		for i in 0..=u16::MAX {
			test_roundtrip(i);
		}
	}

	#[test]
	fn test_decode_boundary_conditions() {
		let examples = [
			([].as_slice(), Err(DecodeError::MissingBytes)),
			(&[0], Ok((0, [].as_slice()))),
			(&[0, 0], Ok((0, &[0]))),
			(&[0, 1], Ok((0, &[1]))),
			(&[0, u8::MAX], Ok((0, &[u8::MAX]))),
			(&[u8::MAX], Err(DecodeError::MissingBytes)),
			(&[1 << 7], Err(DecodeError::MissingBytes)),
			(&[LSB_7], Ok((LSB_7 as u16, &[]))),
			(&[LSB_7, 0], Ok((LSB_7 as u16, &[0]))),
			(&[LSB_7, u8::MAX], Ok((LSB_7 as u16, &[u8::MAX]))),
		];

		for (encoded, result) in examples {
			assert_eq!(decode_varint(encoded), result, "decoded from {encoded:?}");
		}
	}

	fn generate_all_valid_varints() -> HashMap<Vec<u8>, u16> {
		let mut result = HashMap::with_capacity(u16::MAX as usize);
		for i in 0..=u16::MAX {
			let encoded = encode_varint(i);
			result.insert(encoded.as_slice().to_vec(), i);
		}
		result
	}

	const SPECIAL_BYTES: &[u8] =
		&[0, 1, u8::MAX, u8::MAX - 1, LSB_7, LSB_7 + 1, LSB_7 - 1];

	fn generate_all_byte_patterns() -> impl Iterator<Item = Vec<u8>> {
		let mut iterators = Vec::new();
		const LEN: usize = 3;
		for i in 1..=LEN {
			iterators.push((0..=u8::MAX).combinations_with_replacement(i));
		}
		iterators
			.into_iter()
			.flatten()
			.chain([vec![]])
			// We do this instead of increasting the LEN to avoid the test taking too long, since
			// the time is exponential in LEN
			.chain(
				(0..=u8::MAX)
					.combinations_with_replacement(LEN)
					.cartesian_product(SPECIAL_BYTES)
					.map(|(mut bytes, special)| {
						bytes.push(*special);
						bytes
					}),
			)
	}

	#[test]
	fn test_all_varints() {
		let valid_varints = generate_all_valid_varints();
		for bytes in generate_all_byte_patterns() {
			match decode_varint(&bytes) {
				Ok((decoded, tail_bytes)) => {
					let head_bytes = &bytes[0..(bytes.len() - tail_bytes.len())];
					assert_eq!(
						head_bytes,
						encode_varint(decoded).as_slice(),
						"expected head bytes {head_bytes:?} of bytes {bytes:?} to match encoding of decoded value {decoded}"
					);
					if let Some(&expected_decoded) = valid_varints.get(head_bytes) {
						assert_eq!(
							decoded, expected_decoded,
							"expected successful decode of {bytes:?} to match expected decode value"
						);
					} else {
						panic!("expected all successful decodes to have a matching entry in the valid varint set. Offending bytes: {bytes:?}");
					}
				}
				Err(_err) => assert!(!valid_varints.contains_key(&bytes)),
			}
		}
	}
}
