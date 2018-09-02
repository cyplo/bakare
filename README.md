Goals:
* fast
* using max bandwidth
* memory usage limit
* encryption by default - asymmetric, creates a keypair for you
* deduplication
* fuzzy find by file name in stored files

Implementation:
* hash -> file and file -> hash indexes
* use vfs to store both db and data files, create a new one when old one too big
* start with simple read file -> hash -> encrypt -?> send
* test with randomly created dirs and files, with property based tests and fuzzer