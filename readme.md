# reproducible 408 under high load


## Server
Install meilisearch v1.1.0 

config.toml:

```
env="development"
db_path="../../data/meilifile"
master_key="1234567890123456"
http_addr="localhost:7777"
max_indexing_memory = "64GB"
max_indexing_threads = 9
log_level="DEBUG"
```

Meilisearch is running on ubuntu 22 on a server with 128G RAM and 12 cores.

compiled from source using rustc in release mode: 

```
rustc 1.68.2 (9eb3afe9e 2023-03-27)
```

see the ```common/src/lib.rs``` file for host configuration and API key.

## reproduce error

- clone repo
- compile using ```cargo build --release```
- insert test data: run  ```target/release/insert_dummy_data``` - this will insert 10_000_000 documents in the ```movie``` index, 10_000_000 documents
into the ```person``` index and approx. 56_000_000 documents into the ```principal``` index. These are roughly the same numbers as in the original dataset: https://datasets.imdbws.com/
- run the  ```target/release/create_combined_index``` binary. This program will spawn some tokio tasks (4 tasks are enough to provoke the 408 error). Each task reads documents paginated (limited to 100 movies) from the ```movie``` index and
for each movie executes two filter queries: one for the ```principal``` and the second one for the ```person``` index. The program then combines the data into a ```SearchDoc``` document. After all 100 movies are processed the program tries to insert a list of 100 ```SearchDoc``` documents into the ```search_doc``` index.

Running the ```target/release/create_combined_index``` crashes some of the tokio tasks almost immediately after starting the program. 
The Rust meilisearch client (https://github.com/meilisearch/meilisearch-rust) logs a warning: ```meilisearch_sdk  meilisearch_sdk::request] Expected response code 200, got 408```

Log output of the ```target/release/create_combined_index``` binary:

```
target/release/create_combined_index

[2023-04-08T18:51:44Z INFO  create_combined_index] cnt_tasks 4, total_movies 10000000, movies_per_task 2500000, limit 100
[2023-04-08T18:51:44Z INFO  create_combined_index] new thread starts at offset: 0, limit 100
[2023-04-08T18:51:44Z INFO  create_combined_index] new thread starts at offset: 2500000, limit 100
[2023-04-08T18:51:44Z INFO  create_combined_index] new thread starts at offset: 5000000, limit 100
[2023-04-08T18:51:44Z INFO  create_combined_index] new thread starts at offset: 7500000, limit 100
[2023-04-08T18:51:44Z INFO  create_combined_index] found 100 movies
[2023-04-08T18:51:44Z INFO  create_combined_index]  movie tconst: 1  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:46Z INFO  create_combined_index] found 100 movies
[2023-04-08T18:51:48Z INFO  create_combined_index] found 100 movies
[2023-04-08T18:51:48Z INFO  create_combined_index]  movie tconst: 2500001  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:48Z INFO  create_combined_index]  movie tconst: 2500002  -> principals found: 2 found  6 persons using these  principals nconsts: '{"2", "1"}'
[2023-04-08T18:51:48Z INFO  create_combined_index]  movie tconst: 2500003  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:48Z INFO  create_combined_index]  movie tconst: 2500004  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:50Z WARN  meilisearch_sdk::request] Expected response code 200, got 408
[2023-04-08T18:51:50Z INFO  create_combined_index] found 100 movies
[2023-04-08T18:51:50Z INFO  create_combined_index] error reading persons Error parsing response JSON: invalid type: null, expected struct MeilisearchError at line 1 column 4
thread 'tokio-runtime-worker' panicked at 'should not happen. error reading persons.', create_combined_index/src/main.rs:142:13
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[2023-04-08T18:51:50Z WARN  meilisearch_sdk::request] Expected response code 200, got 408
[2023-04-08T18:51:50Z INFO  create_combined_index] error reading persons Error parsing response JSON: invalid type: null, expected struct MeilisearchError at line 1 column 4
thread 'tokio-runtime-worker' panicked at 'should not happen. error reading persons.', create_combined_index/src/main.rs:142:13
[2023-04-08T18:51:50Z WARN  meilisearch_sdk::request] Expected response code 200, got 408
thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an `Err` value: ParseError(Error("invalid type: null, expected struct MeilisearchError", line: 1, column: 4))', create_combined_index/src/main.rs:118:10
[2023-04-08T18:51:50Z ERROR create_combined_index] task crashed and returned an error task 9 panicked
[2023-04-08T18:51:50Z ERROR create_combined_index] task crashed and returned an error task 10 panicked
[2023-04-08T18:51:50Z ERROR create_combined_index] task crashed and returned an error task 11 panicked
[2023-04-08T18:51:50Z INFO  create_combined_index]  movie tconst: 7500001  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:50Z INFO  create_combined_index]  movie tconst: 7500002  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:50Z INFO  create_combined_index]  movie tconst: 7500003  -> principals found: 2 found  6 persons using these  principals nconsts: '{"2", "1"}'
[2023-04-08T18:51:50Z INFO  create_combined_index]  movie tconst: 7500004  -> principals found: 2 found  6 persons using these  principals nconsts: '{"1", "2"}'
[2023-04-08T18:51:51Z INFO  create_combined_index]  movie tconst: 7500005  -> principals found: 2 found  6 persons using these  principals nconsts: '{"2", "1"}'

```

Each filter query from the ```principal```  index return 2 documents, the filter query from the  ```person``` index always returns 6 documents.

The index sizes are the "smallest" sizes where the problem could be reproduced. 
Smaller datasets and for example increasing the number of spawned tasks could not trigger the 408 response. 


## Output from meilisearch server start

```
8888b   d8888          Y8P 888 Y8P                                            888
88888b.d88888              888                                                888
888Y88888P888  .d88b.  888 888 888 .d8888b   .d88b.   8888b.  888d888 .d8888b 88888b.
888 Y888P 888 d8P  Y8b 888 888 888 88K      d8P  Y8b     "88b 888P"  d88P"    888 "88b
888  Y8P  888 88888888 888 888 888 "Y8888b. 88888888 .d888888 888    888      888  888
888   "   888 Y8b.     888 888 888      X88 Y8b.     888  888 888    Y88b.    888  888
888       888  "Y8888  888 888 888  88888P'  "Y8888  "Y888888 888     "Y8888P 888  888

Config file path:       "./config.toml"
Database path:          "../../data/meilifile"
Server listening on:    "http://localhost:7777"
Environment:            "development"
Commit SHA:             "950f73b8bbc85c4e5c3485d7054ff697cb0c0052"
Commit date:            "2023-03-30T08:31:29Z"
Package version:        "1.1.0"

Thank you for using Meilisearch!


We collect anonymized analytics to improve our product and your experience. To learn more, including how to turn off analytics, visit our dedicated documentation page: https://docs.meilisearch.com/learn/what_is_meilisearch/telemetry.html

Anonymous telemetry:    "Enabled"
Instance UID:           "9bcdeece-0459-45f0-ba8f-67d9e2c26cac"

A master key has been set. Requests to Meilisearch won't be authorized unless you provide an authentication key.

Documentation:          https://docs.meilisearch.com
Source code:            https://github.com/meilisearch/meilisearch
Contact:                https://docs.meilisearch.com/resources/contact.html

[2023-04-08T19:21:43Z INFO  actix_server::builder] Starting 6 workers
[2023-04-08T19:21:43Z INFO  actix_server::server] Actix runtime found; starting in Actix runtime
```