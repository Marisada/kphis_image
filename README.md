# image module implementation test for KPHIS

## aim
- universal module for diffenent scenario
- build with viewer
- can select and copy/paste/delete
- can clear unused image

## archetecture

### file 
- resize and create thumbnail at client side
- saving with `webp` format
- seperate `images` and `thumbs` main directories, using the same sub-directory tree
- `01JG0M004KYHATX7J2W7MB28X4` Ulid will using path like `01J/G0/M004KYHATX7J2W7MB28X4.webp`
> - first 10 chars of Ulid is timestamp(ms)  
> - 3rd Ulid char step up every 397 days and 16 hours  
> - 5th Ulid char step up every 9 hours and 20 minutes  
> - 2 chars of Ulid has 32x32=1024 possible folders  
> - amount of sub-directiries will be as `yearly` / `max 1024` / `within 9 hours`
- so image `file` path will be `volume/images/01J/G0/M004KYHATX7J2W7MB28X4.webp` and `url` path will be `images/01J/G0/M004KYHATX7J2W7MB28X4.webp`
- and thumbnail `file` path will be `volume/thumbs/01J/G0/M004KYHATX7J2W7MB28X4.webp` and `url` path will be `thumbs/01J/G0/M004KYHATX7J2W7MB28X4.webp`
## database
1. primary key 
    - [x] `UNSIGNED INT`(4 bytes, u32 max~4.29x10^9) (can step to `UNSIGNED BIGINT`(8 bytes, u64 max~18.44x10^18) later)
    - [ ] `BINARY(16)` of `Ulid`(u128) (`BINARY(16)` has a better performance than `CHAR(26)`) 
> - PostgreSQL uses big-endian, and MySQL uses little-endian  
> - RUST can convert `Ulid(u128)` to `u128` and from `u128` to `[u8;16]` with `u128::to_be_bytes()` and `u128::to_le_bytes()`
> - Read more about MySQL performance comparison between INT/BIGINT/UUID/ULID key at [medium.com](https://medium.com/@dariusmatonas/mysql-uuid-vs-ulid-vs-int-bb6083bfd6cf)  
> - webp image 1024x1024 size is 1115KB + thumbnail 144x144 size is 32KB, total 1147KB per id, so 4.29x10^9 ids need 4920TB of storage
2. table relation
    1. [ ] single table, without 1:1 file:row table
        - has duplicate path, so CANNOT health check with file storage
        - has `image_id`, `mode`, `mode_id`, `path`, `title`, `create_user`, `create_datetime` columns
    2. [ ] table for each xxx mode, without 1:1 file:row table
        - has duplicate path across each mode, so CANNOT health check with file storage
        - has `image_id`, `xxx_id`, `path`, `title`, `create_user`, `create_datetime` columns
    3. [x] with 1:1 file:row table
        - CAN health check with file storage by images table
        - images table has `image_id`, `path`, `title`, `create_user`, `create_datetime` columns
        - [ ] single table has `image_usage_id`, `use_at`, `at_id`, `image_id`, `create_user`, `create_datetime` columns
        - [ ] xxx mode table has `image_usage_id`, `xxx_id`, `image_id`, `create_user`, `create_datetime` 
        
3. health check 
- if file was loss, db entry MUST be deleted
- if db entry was less, file MUST be deleted

```rust
let from_db: Vec<Path> = get_path_start_with("01J/G0").await.unwrap();
let from_fs: Vec<Path> = fs::read_dir("/volume/thumbs/01J/G0").unwrap().filter_map(|dir_entry| {
    dir_entry.ok().map(|entry| {
        entry.path()
    })
}).collect();
// bruteforce method
let mut missed_in_fs = Vec::new();
let mut missed_in_db = Vec::new();
for db_path in from_db.iter() {
    if !from_fs.contains(&db_path) {
        missed_in_fs.push(db_path);
    }
}
for fs_path in from_fs.iter() {
    if !from_db.contains(&fs_path) {
        missed_in_db.push(fs_path);
    }
} 
// hashset method
// TODO
```