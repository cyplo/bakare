use std::io::Read;

use anyhow::*;
use reed_solomon::Encoder;
use reed_solomon::{Buffer, Decoder};

const BLOCK_SIZE: usize = 255 - ECC_LENGTH;
const ECC_LENGTH: usize = 8;

// TODO: make the API streaming friendly
pub fn encode(bytes: &[u8]) -> Result<Vec<u8>> {
    let encoder = Encoder::new(ECC_LENGTH);

    let encoded_blocks = bytes
        .chunks(BLOCK_SIZE)
        .map(|chunk| encoder.encode(chunk))
        .collect::<Vec<Buffer>>();

    let mut result = vec![];

    for buffer in encoded_blocks {
        for byte in buffer.bytes() {
            result.push(byte?);
        }
    }

    Ok(result)
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let decoder = Decoder::new(ECC_LENGTH);
    let decoded_blocks = bytes
        .chunks(BLOCK_SIZE + ECC_LENGTH)
        .map(|chunk| decoder.correct(chunk, None).map_err(|e| anyhow!(format!("{:#?}", e))))
        .collect::<Result<Vec<Buffer>>>()?;

    let mut result = vec![];

    for buffer in decoded_blocks {
        for byte in buffer.data() {
            result.push(*byte);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod must {

    use anyhow::Result;
    use rand::{thread_rng, Rng, RngCore};

    use super::{decode, encode};

    use pretty_assertions::assert_eq;

    #[test]
    fn encode_small_amounts_of_data() -> Result<()> {
        let mut original: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut original);

        let decoded = decode(&encode(&original)?)?;

        assert_eq!(decoded, original);

        Ok(())
    }
    #[test]
    fn encode_large_amounts_of_data() -> Result<()> {
        let mut original: [u8; 1024 * 1024] = [0; 1024 * 1024];
        thread_rng().fill_bytes(&mut original);

        let decoded = decode(&encode(&original)?)?;

        assert_eq!(decoded, original);

        Ok(())
    }

    #[test]
    fn survive_data_corruption() -> Result<()> {
        let mut original: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut original);

        let encoded = encode(&original)?;

        let size = encoded.len();
        let corrupt_byte_index = rand::thread_rng().gen_range::<usize, _>(0..size);

        let mut corrupted = encoded;
        corrupted[corrupt_byte_index] = rand::thread_rng().gen::<u8>();

        let decoded = decode(&corrupted)?;

        assert_eq!(decoded, original);

        Ok(())
    }
}
