use crate::{symmetric::message_hash::bytes_to_chunks, MESSAGE_LENGTH};

use super::MessageHash;

use sha2::{Digest, Sha256};

/// A message hash implemented using SHA-256
/// All lengths must be given in Bytes.
/// All lengths must be less than 255 bits.
/// Randomness length must be non-zero.
/// CHUNK_SIZE has to be 1,2,4, or 8.
pub struct Sha256MessageHash<
    const PARAMETER_LEN: usize,
    const RAND_LEN: usize,
    const NUM_CHUNKS: usize,
    const CHUNK_SIZE: usize,
>;

impl<
        const PARAMETER_LEN: usize,
        const RAND_LEN: usize,
        const NUM_CHUNKS: usize,
        const CHUNK_SIZE: usize,
    > MessageHash for Sha256MessageHash<PARAMETER_LEN, RAND_LEN, NUM_CHUNKS, CHUNK_SIZE>
{
    type Parameter = [u8; PARAMETER_LEN];

    type Randomness = [u8; RAND_LEN];

    const NUM_CHUNKS: usize = NUM_CHUNKS;

    const CHUNK_SIZE: usize = CHUNK_SIZE;

    fn rand<R: rand::Rng>(rng: &mut R) -> Self::Randomness {
        let mut rand = [0u8; RAND_LEN];
        rng.fill_bytes(&mut rand);
        rand
    }

    fn apply(
        parameter: &Self::Parameter,
        epoch: u32,
        randomness: &Self::Randomness,
        message: &[u8; MESSAGE_LENGTH],
    ) -> Vec<u8> {
        assert!(
            PARAMETER_LEN < 256 / 8,
            "SHA256-Message Hash: Parameter Length must be less than 256 bit"
        );
        assert!(
            RAND_LEN < 256 / 8,
            "SHA256-Message Hash: Randomness Length must be less than 256 bit"
        );
        assert!(
            RAND_LEN > 0,
            "SHA256-Message Hash: Randomness Length must be non-zero"
        );
        assert!(
            NUM_CHUNKS * CHUNK_SIZE < 256,
            "SHA256-Message Hash: Hash Length (= NUM_CHUNKS * CHUNK_SIZE) must be less than 256 bit"
        );

        let mut hasher = Sha256::new();

        // now add the parameter
        hasher.update(parameter);

        // now add tweak (= domain separator + epoch)
        // domain separater: this is a message hash tweak.
        // So we start with a 0x02 byte.
        hasher.update(&[0x02]);
        hasher.update(epoch.to_le_bytes());

        // now add randomness
        hasher.update(randomness);

        // now add the actual message to be hashed
        hasher.update(message);

        // finalize the hash, and take as many bytes as we need
        let hash = hasher.finalize();
        // turn the bytes in the hash into chunks
        let chunks: Vec<u8> =
            bytes_to_chunks(&hash[0..NUM_CHUNKS * CHUNK_SIZE / 8], Self::CHUNK_SIZE);
        chunks
    }
}

// Example instantiations
pub type Sha256MessageHash128x3 = Sha256MessageHash<16, 16, 16, 8>;
pub type Sha256MessageHash192x3 = Sha256MessageHash<24, 24, 48, 4>;

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use crate::MESSAGE_LENGTH;

    use super::*;

    #[test]
    fn test_apply_128x3() {
        let mut rng = thread_rng();

        let mut parameter = [0u8; 16];
        rng.fill(&mut parameter);

        let mut message = [0u8; MESSAGE_LENGTH];
        rng.fill(&mut message);

        let epoch = 13;
        let randomness = Sha256MessageHash128x3::rand(&mut rng);

        Sha256MessageHash128x3::apply(&parameter, epoch, &randomness, &message);
    }

    #[test]
    fn test_apply_192x3() {
        let mut rng = thread_rng();

        let mut parameter = [0u8; 24];
        rng.fill(&mut parameter);

        let mut message = [0u8; MESSAGE_LENGTH];
        rng.fill(&mut message);

        let epoch = 13;
        let randomness = Sha256MessageHash192x3::rand(&mut rng);

        Sha256MessageHash192x3::apply(&parameter, epoch, &randomness, &message);
    }
}
