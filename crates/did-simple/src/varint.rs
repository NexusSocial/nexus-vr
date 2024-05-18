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
	buf: [u8; 3],
	len: u8,
}

impl VarintEncoding {
	#[allow(dead_code)]
	pub const fn as_slice(&self) -> &[u8] {
		self.buf.split_at(self.len as usize).0
	}
}

/// Encodes a value as a varint.
/// Returns an array as well as the length of the array to slice., along  well as an array.
#[allow(dead_code)]
pub(crate) const fn encode_varint(value: u16) -> VarintEncoding {
	let mut out_buf = [0; 3];
	// ilog2 can be used to get the (0-indexed from LSB position) of the MSB.
	// We then add one to it, since we are trying to indicate length, not position.
	let in_bit_length: u16 = if let Some(x) = value.checked_ilog2() {
		(x + 1) as u16
	} else {
		return VarintEncoding {
			buf: out_buf,
			len: 0,
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

pub(crate) const fn decode_varint(encoded: &[u8]) -> Result<u16, DecodeError> {
	// TODO: Technically, some three byte encodings could fit into a u16, we
	// should support those in the future.
	// Luckily none of them are used for did:key afaik.
	if encoded.len() > 2 {
		return Err(DecodeError::WouldOverflow);
	}
	if encoded.is_empty() {
		return Err(DecodeError::MissingBytes);
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
			assert_eq!(Ok(decoded), decode_varint(encoded));
			assert_eq!(encoded, encode_varint(decoded).as_slice());
		}
	}
}
