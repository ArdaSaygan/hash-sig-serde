// use num_bigint::BigUint;
// use zkhash::ark_ff::MontConfig;
// use zkhash::ark_ff::PrimeField;
// use zkhash::ark_ff::UniformRand;
// use zkhash::ark_ff::Zero;
// use zkhash::fields::babybear::FpBabyBear;
// use zkhash::fields::babybear::FqConfig;
// use zkhash::poseidon2::poseidon2::Poseidon2;
// use zkhash::poseidon2::poseidon2_instance_babybear::POSEIDON2_BABYBEAR_24_PARAMS;

// use super::MessageHash;
// use crate::symmetric::tweak_hash::poseidon::poseidon_compress;
// use crate::MESSAGE_LENGTH;
// use crate::TWEAK_SEPARATOR_FOR_MESSAGE_HASH;

// type F = FpBabyBear;

// /// Function to encode a message as an array of field elements
// fn encode_message<const MSG_LEN_FE: usize>(message: &[u8; MESSAGE_LENGTH]) -> [F; MSG_LEN_FE] {
//     // Interpret message as a little-endian integer
//     let mut acc = BigUint::from_bytes_le(message);

//     // Get the modulus as BigUint once
//     let p = BigUint::from(FqConfig::MODULUS);

//     // Perform base-p decomposition
//     std::array::from_fn(|_| {
//         let digit = &acc % &p;
//         acc /= &p;
//         F::from(digit)
//     })
// }

// /// Function to encode an epoch (= tweak in the message hash)
// /// as a vector of field elements.
// fn encode_epoch<const TWEAK_LEN_FE: usize>(epoch: u32) -> [F; TWEAK_LEN_FE] {
//     // convert the bytes (together with domain separator) into a number
//     let epoch_uint: BigUint = (BigUint::from(epoch) << 8) + TWEAK_SEPARATOR_FOR_MESSAGE_HASH;

//     // now interpret the number in base-p
//     let mut tweak_fe: [F; TWEAK_LEN_FE] = [F::zero(); TWEAK_LEN_FE];
//     tweak_fe.iter_mut().fold(epoch_uint, |acc, item| {
//         let tmp = acc.clone() % BigUint::from(FqConfig::MODULUS);
//         *item = F::from(tmp.clone());
//         (acc - tmp) / (BigUint::from(FqConfig::MODULUS))
//     });
//     tweak_fe
// }

// /// Function to decode a vector of field elements into
// /// a vector of NUM_CHUNKS many chunks. One chunk is
// /// between 0 and 2^CHUNK_SIZE - 1 (inclusive).
// /// CHUNK_SIZE up to 8 (inclusive) is supported
// fn decode_to_chunks<const NUM_CHUNKS: usize, const CHUNK_SIZE: usize, const HASH_LEN_FE: usize>(
//     field_elements: &[F; HASH_LEN_FE],
// ) -> Vec<u8> {
//     // Turn field elements into a big integer
//     let hash_uint = field_elements.iter().fold(BigUint::ZERO, |acc, &item| {
//         acc * BigUint::from(FqConfig::MODULUS) + BigUint::from(item.into_bigint())
//     });

//     // Split the integer into chunks
//     let max_chunk_len = (1 << CHUNK_SIZE) as u16;

//     let mut hash_chunked: [u8; NUM_CHUNKS] = [0; NUM_CHUNKS];
//     hash_chunked.iter_mut().fold(hash_uint, |acc, item| {
//         *item = (acc.clone() % max_chunk_len).to_bytes_be()[0];
//         (acc - *item) / max_chunk_len
//     });
//     Vec::from(hash_chunked)
// }

// /// A message hash implemented using Poseidon2
// ///
// /// Note: PARAMETER_LEN, RAND_LEN, TWEAK_LEN_FE, MSG_LEN_FE, and HASH_LEN_FE
// /// must be given in the unit "number of field elements".
// ///
// /// HASH_LEN_FE specifies how many field elements the
// /// hash output needs to be before it is decoded to chunks.
// ///
// /// CHUNK_SIZE has to be 1,2,4, or 8.
// pub struct PoseidonMessageHash<
//     const PARAMETER_LEN: usize,
//     const RAND_LEN: usize,
//     const HASH_LEN_FE: usize,
//     const NUM_CHUNKS: usize,
//     const CHUNK_SIZE: usize,
//     const TWEAK_LEN_FE: usize,
//     const MSG_LEN_FE: usize,
// >;

// impl<
//         const PARAMETER_LEN: usize,
//         const RAND_LEN: usize,
//         const HASH_LEN_FE: usize,
//         const NUM_CHUNKS: usize,
//         const CHUNK_SIZE: usize,
//         const TWEAK_LEN_FE: usize,
//         const MSG_LEN_FE: usize,
//     > MessageHash
//     for PoseidonMessageHash<
//         PARAMETER_LEN,
//         RAND_LEN,
//         HASH_LEN_FE,
//         NUM_CHUNKS,
//         CHUNK_SIZE,
//         TWEAK_LEN_FE,
//         MSG_LEN_FE,
//     >
// {
//     type Parameter = [F; PARAMETER_LEN];

//     type Randomness = [F; RAND_LEN];

//     const NUM_CHUNKS: usize = NUM_CHUNKS;

//     const CHUNK_SIZE: usize = CHUNK_SIZE;

//     fn rand<R: rand::Rng>(rng: &mut R) -> Self::Randomness {
//         std::array::from_fn(|_| F::rand(rng))
//     }

//     fn apply(
//         parameter: &Self::Parameter,
//         epoch: u32,
//         randomness: &Self::Randomness,
//         message: &[u8; MESSAGE_LENGTH],
//     ) -> Vec<u8> {
//         // We need a Poseidon instance

//         // Note: This block should be changed if we decide to support other Poseidon
//         // instances. Currently we use state of width 24 and pad with 0s.
//         assert!(PARAMETER_LEN + TWEAK_LEN_FE + RAND_LEN + MSG_LEN_FE <= 24);
//         let instance = Poseidon2::new(&POSEIDON2_BABYBEAR_24_PARAMS);

//         // first, encode the message and the epoch as field elements
//         let message_fe = encode_message::<MSG_LEN_FE>(message);
//         let epoch_fe = encode_epoch::<TWEAK_LEN_FE>(epoch);

//         // now, we hash randomness, parameters, epoch, message using PoseidonCompress
//         let combined_input: Vec<F> = randomness
//             .iter()
//             .chain(parameter.iter())
//             .chain(epoch_fe.iter())
//             .chain(message_fe.iter())
//             .copied()
//             .collect();
//         let hash_fe = poseidon_compress::<HASH_LEN_FE>(&instance, &combined_input);

//         // decode field elements into chunks and return them
//         decode_to_chunks::<NUM_CHUNKS, CHUNK_SIZE, HASH_LEN_FE>(&hash_fe)
//     }

//     #[cfg(test)]
//     fn internal_consistency_check() {
//         // message check
//         let message_fe_bits = f64::log2(
//             BigUint::from(FqConfig::MODULUS)
//                 .to_string()
//                 .parse()
//                 .unwrap(),
//         ) * f64::from(MSG_LEN_FE as u32);
//         assert!(
//             message_fe_bits >= f64::from((8_u32) * (MESSAGE_LENGTH as u32)),
//             "Poseidon Message hash. Parameter mismatch: not enough field elements to encode the message"
//         );

//         // tweak check
//         let tweak_fe_bits = f64::log2(
//             BigUint::from(FqConfig::MODULUS)
//                 .to_string()
//                 .parse()
//                 .unwrap(),
//         ) * f64::from(TWEAK_LEN_FE as u32);
//         assert!(
//             tweak_fe_bits >= f64::from(32 + 8_u32),
//             "Poseidon Message hash. Parameter mismatch: not enough field elements to encode the epoch tweak"
//         );

//         // decoding check
//         let hash_bits = f64::log2(
//             BigUint::from(FqConfig::MODULUS)
//                 .to_string()
//                 .parse()
//                 .unwrap(),
//         ) * f64::from(HASH_LEN_FE as u32);
//         assert!(
//             hash_bits <= f64::from((NUM_CHUNKS * CHUNK_SIZE) as u32),
//             "Poseidon Message hash. Parameter mismatch: not enough chunks to decode the hash"
//         );
//     }
// }

// // Example instantiations
// pub type PoseidonMessageHash445 = PoseidonMessageHash<4, 4, 5, 128, 2, 2, 9>;
// pub type PoseidonMessageHashW1 = PoseidonMessageHash<5, 5, 5, 163, 1, 2, 9>;

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rand::{thread_rng, Rng};
//     use zkhash::ark_ff::Field;
//     use zkhash::ark_ff::One;
//     use zkhash::ark_ff::UniformRand;

//     #[test]
//     fn test_apply() {
//         let mut rng = thread_rng();

//         let mut parameter = [F::one(); 4];
//         for p in &mut parameter {
//             *p = F::rand(&mut rng);
//         }

//         let mut message = [0u8; MESSAGE_LENGTH];
//         rng.fill(&mut message);

//         let epoch = 13;
//         let randomness = PoseidonMessageHash445::rand(&mut rng);

//         PoseidonMessageHash445::internal_consistency_check();
//         PoseidonMessageHash445::apply(&parameter, epoch, &randomness, &message);
//     }
//     #[test]
//     fn test_apply_w1() {
//         let mut rng = thread_rng();

//         let mut parameter = [F::one(); 5];
//         for p in &mut parameter {
//             *p = F::rand(&mut rng);
//         }

//         let mut message = [0u8; MESSAGE_LENGTH];
//         rng.fill(&mut message);

//         let epoch = 13;
//         let randomness = PoseidonMessageHashW1::rand(&mut rng);

//         PoseidonMessageHashW1::internal_consistency_check();
//         PoseidonMessageHashW1::apply(&parameter, epoch, &randomness, &message);
//     }

//     #[test]
//     fn test_rand_not_all_same() {
//         let mut rng = thread_rng();
//         // Setup a number of trials
//         const K: usize = 10;
//         let mut all_same_count = 0;

//         for _ in 0..K {
//             let randomness = PoseidonMessageHash445::rand(&mut rng);

//             // Check if all elements in `randomness` are identical
//             let first = randomness[0];
//             if randomness.iter().all(|&x| x == first) {
//                 all_same_count += 1;
//             }
//         }

//         // If all K trials resulted in identical values, fail the test
//         assert!(
//             all_same_count < K,
//             "rand generated identical elements in all {} trials",
//             K
//         );
//     }

//     #[test]
//     fn test_encode_message_all_zeros() {
//         // Message
//         let message = [0u8; 32];

//         // Expected = 9 zeros, as 9 * 31 >= 8 * 32
//         let expected = [F::ZERO; 9];

//         let computed = super::encode_message::<9>(&message);
//         assert_eq!(computed, expected);
//     }

//     #[test]
//     fn test_encode_message_all_max() {
//         // Message
//         let message = [u8::MAX; 32];

//         // Convert to bigint
//         let message_bigint = BigUint::from_bytes_le(&message);

//         // Field modulus
//         let p = BigUint::from(FqConfig::MODULUS);

//         // Compute expected: base-p decomposition
//         //
//         // We compute this by hand to ensure that the test is correct.
//         let expected = [
//             F::from(&message_bigint % &p),
//             F::from((&message_bigint / &p) % &p),
//             F::from((&message_bigint / (&p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p * &p * &p)) % &p),
//         ];

//         let computed = super::encode_message::<9>(&message);
//         assert_eq!(computed, expected);
//     }

//     #[test]
//     fn test_encode_message_mixed_bytes() {
//         // Alternating 0x00 and 0xFF
//         let mut message = [0u8; 32];
//         for (i, byte) in message.iter_mut().enumerate() {
//             *byte = if i % 2 == 0 { 0x00 } else { 0xFF };
//         }

//         // Convert to bigint
//         let message_bigint = BigUint::from_bytes_le(&message);

//         // Field modulus
//         let p = BigUint::from(FqConfig::MODULUS);

//         // Compute expected: base-p decomposition
//         //
//         // We compute this by hand to ensure that the test is correct.
//         let expected = [
//             F::from(&message_bigint % &p),
//             F::from((&message_bigint / &p) % &p),
//             F::from((&message_bigint / (&p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p * &p)) % &p),
//             F::from((&message_bigint / (&p * &p * &p * &p * &p * &p * &p * &p)) % &p),
//         ];

//         let computed = super::encode_message::<9>(&message);
//         assert_eq!(computed, expected);
//     }
// }
