use anyhow::*;

pub fn encode(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(Vec::from(bytes))
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    Ok(Vec::from(bytes))
}

#[cfg(test)]
mod must {

    use anyhow::Result;
    use rand::{thread_rng, Rng, RngCore};

    use super::{decode, encode};

    use pretty_assertions::assert_eq;

    #[test]
    #[ignore = "wip"]
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
