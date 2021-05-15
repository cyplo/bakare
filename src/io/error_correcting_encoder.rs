use std::io::Read;

use anyhow::*;
use reed_solomon::Decoder;
use reed_solomon::Encoder;

const ECC_LENGTH: usize = 8;

pub fn encode(bytes: &[u8]) -> Result<Vec<u8>> {
    let encoder = Encoder::new(ECC_LENGTH);
    let encoded = encoder.encode(bytes);
    Ok(encoded.bytes().collect::<Result<Vec<u8>, _>>()?)
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let decoder = Decoder::new(ECC_LENGTH);
    if decoder.is_corrupted(bytes) {
        return Err(anyhow!("corrupted"));
    }
    let maybe_corrected = decoder.correct(bytes, None);
    match maybe_corrected {
        Ok(corrected) => Ok(corrected.data().to_vec()),
        Err(_) => Err(anyhow!("")),
    }
}

#[cfg(test)]
mod must {

    use anyhow::Result;
    use rand::{thread_rng, Rng, RngCore};

    use super::{decode, encode};

    use pretty_assertions::assert_eq;

    #[test]
    fn survive_data_corruption() -> Result<()> {
        let mut original: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut original);

        let encoded = encode(&original)?;

        let size = encoded.len();
        let corrupt_byte_index = rand::thread_rng().gen_range::<usize, _>(0..size);

        let mut corrupted = Vec::from(encoded);
        corrupted[corrupt_byte_index] = rand::thread_rng().gen::<u8>();

        let decoded = decode(&corrupted).unwrap();

        assert_eq!(decoded, original);

        Ok(())
    }
}
