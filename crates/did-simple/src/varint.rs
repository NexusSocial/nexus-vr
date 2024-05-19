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

	pub const fn as_slice(&self) -> &[u8] {
		self.buf.split_at(self.len as usize).0
	}
}

/// Encodes a value as a varint.
/// Returns an array as well as the length of the array to slice., along  well as an array.
pub(crate) const fn encode_varint(value: u16) -> VarintEncoding {
	let mut out_buf = [0; VarintEncoding::MAX_LEN];
	// ilog2 can be used to get the (0-indexed from LSB position) of the MSB.
	// We then add one to it, since we are trying to indicate length, not position.
	let in_bit_length: u16 = if let Some(x) = value.checked_ilog2() {
		(x + 1) as u16
	} else {
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
		let Some(shifted) =
			((current_byte & LSB_7) as u16).checked_shl(current_decoded_bit)
		else {
			return Err(DecodeError::WouldOverflow);
		};
		decoded |= shifted;
		current_decoded_bit += 7;
		current_encoded_idx += 1;
		if !msb_is_1(current_byte) {
			break;
		}
	}
	let bytes_in_varint = current_encoded_idx;

	Ok((decoded, encoded.split_at(bytes_in_varint).1))
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

	fn test_roundtrip(decoded: u16) {
		let extra_bytes = [1, 2, 3, 4].as_slice();
		let empty = [].as_slice();
		let encoded = encode_varint(decoded);
		println!("encoded {:?}", encoded.as_slice());

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
		for i in 0..u16::MAX {
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
}
