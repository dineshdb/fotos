# Files

A deduplicated file manager using EROFS and ComposeFS like mechanism. Difference between this and composefs is that it allows modifying the filesystem.

## Features

- FS-verity and data verification on read
- Automatic file level deduplication
- High performance due to erofs overlay mounting (read-only). EROFS mounts are readonly. We need to make the write requests go through fuse layer to make it work. After each write, we need to generate a new erofs image. However, the erofs contains only metadata and hence will be only few mbs. All the mounted fs should be remounted after the regeneration.

## Non-goals
- Versioned filesystem for backups
