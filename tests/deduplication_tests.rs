#[cfg(test)]
mod must {
    use bakare::test::assertions::in_memory::*;
    use bakare::{repository::Repository, test::source::TestSource};
    use proptest::prelude::*;
    use tempfile::tempdir;

    proptest! {
        #[test]
        fn store_duplicated_files_just_once(contents in any::<[u8;3]>()) {
            let source = TestSource::new().unwrap();
            let dir = tempdir().unwrap();
            let repository_path = dir.path();
            Repository::init(&repository_path).unwrap();
            assert_eq!(data_weight(&repository_path).unwrap(), 0);

            backup_file_with_byte_contents(&source, &repository_path, "1", &contents).unwrap();
            let first_weight = data_weight(&repository_path).unwrap();
            assert!(first_weight > 0);

            backup_file_with_byte_contents(&source, &repository_path, "2", &contents).unwrap();
            let second_weight = data_weight(&repository_path).unwrap();
            assert_eq!(first_weight, second_weight);

            assert_restored_file_contents(&repository_path, &source.file_path("1").unwrap(), &contents).unwrap();
            assert_restored_file_contents(&repository_path, &source.file_path("2").unwrap(), &contents).unwrap();
        }
    }
}
