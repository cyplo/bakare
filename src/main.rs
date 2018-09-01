extern crate tempfile;

mod rustback {

    use super::*;

    #[cfg(test)]
    mod should {

        use super::*;
        use tempfile::tempdir;
        use std::fs::File;
        use std::io::{self, Write};
        use std::io::Error;

        #[test]
        fn be_able_to_restore_backed_up_files() -> Result<(), Error> {

            let dir = tempdir()?;

            let file_path = dir.path().join("my-temporary-note.txt");
            let mut file = File::create(file_path)?;
            writeln!(file, "Brian was here. Briefly.")?;

            // create a new temp folder
            // add 3 files, two identical content
            // remember file hashes
            let source_path = dir.path();

            //create a new temp folder
            let destination_path = "";
            //let walker = FilesystemWalker::new(source_path);
            //let engine = Engine::with(walker);

            //engine.backup();
            //engine.restore(destination_path);

            // assert on number and hashes of files
            Ok(())
        }

    }

}

mod index {
    use super::*;

    #[cfg(test)]
    mod should {

        #[test]
        fn recognize_files_of_same_contents() {
            assert!(false);
        }

    }

}
