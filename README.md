Motivation:
All the backup systems are either slow or crashing or both on my backup.

Tried duply:

<details>
<summary>Works but is very slow</summary>

```
--------------[ Backup Statistics ]--------------
StartTime 1547198362.85 (Fri Jan 11 09:19:22 2019)
EndTime 1547209509.04 (Fri Jan 11 12:25:09 2019)
ElapsedTime 11146.19 (3 hours 5 minutes 46.19 seconds)
SourceFiles 3065438
SourceFileSize 585041709586 (545 GB)
NewFiles 0
NewFileSize 0 (0 bytes)
DeletedFiles 0
ChangedFiles 0
ChangedFileSize 0 (0 bytes)
ChangedDeltaSize 0 (0 bytes)
DeltaEntries 0
RawDeltaSize 0 (0 bytes)
TotalDestinationSizeChange 111 (111 bytes)
Errors 0
-------------------------------------------------

--- Finished state OK at 12:25:15.000 - Runtime 03:06:43.000 ---
```

</details>

Tried restic:

- crashes with OOM
- corrupts repo if you suspend one backup process and launch another from a different computer

  Goals for bakare:

- fast
- using max bandwidth
- use max cpu
- use max disk I/O
- memory usage limit
- encryption by default - asymmetric, creates a keypair for you
- deduplication of file data
- fuzzy find by file name in stored files
- failure to process one file should not affect any other files
- intermittent network failures should not make the whole process fail (test with random packet drop)
- system suspend/resume should not make the repo become corrupted, even in the presence of other active backup processes running on other computers, targeting same repo - this is where `restic` fails

  Nice to have:

- daemon that listens for file events and updates a list of files to be backed up on the next backup run - or a `continous backup` mode - the daemon uploads the file whenever it sees the change
- peer2peer mode - people storing encrypted backups for each other
- relay mode, where daemon works on one or more central points with local storage (e.g. NAS) and various computers sync with that central location. Then though the central locaiton uploads everything to the other location, typically the cloud.

  Implementation:

- automatic node discovery - two roles: data publisher and data persister - should be able to figure out which node is which automatically
- test with randomly created dirs and files, with property based tests and fuzzer
- see if we can use `salsa` for recomputaiton
- index corruption tests - mutate random byte and see if everything is readable
- network packet drop tests
- use bevy for decoupling ?
- remove all `unwraps`
