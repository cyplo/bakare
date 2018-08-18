

mod index {
    use super::*;

    #[cfg(test)]
    mod should {

        #[test]
        fn recognize_files_of_same_contents() {
            let file_source = FileSource::new();

            file_source.add("file name 1", "file contents 1");
            file_source.add("file name 2", "file contents 1");
            file_source.add("file name 3", "file contents 3");

            let contents_hasher = FakeContentsHasher::default();
            

            let index = Index::with(file_source, contents_hasher);


        }

    }

}
