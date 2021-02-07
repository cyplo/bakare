use anyhow::Result;

pub fn encode(bytes: &[u8]) -> Result<&[u8]> {
    // find number of shards and parity blocks
    // encode checksum with each block
    // when decoding, remove blocks with invalid checksum

    Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<&[u8]> {
    // find number of shards and parity blocks
    // encode checksum with each block
    // when decoding, remove blocks with invalid checksum

    Ok(bytes)
}

mod must {

    use anyhow::Result;
    use rand::{thread_rng, Rng, RngCore};
    use vfs::{MemoryFS, VfsPath};

    use super::{decode, encode};

    #[test]
    fn survive_data_corruption() -> Result<()> {
        let mut original: [u8; 32] = [0; 32];
        thread_rng().fill_bytes(&mut original);

        let encoded = encode(&original)?;

        let size = dbg!(encoded.len());
        let corrupt_byte_index = rand::thread_rng().gen_range::<usize, _>(0..size);

        let mut corrupted: [u8; 32] = [0; 32];
        corrupted.copy_from_slice(encoded);
        corrupted[corrupt_byte_index] = rand::thread_rng().gen::<u8>();

        let decoded = decode(&corrupted)?;

        assert_eq!(decoded, original);

        Ok(())
    }
}
