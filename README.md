# tantivy_warc_indexer

tantivy_warc_indexer builds a [tantivy](https://github.com/tantivy-search/tantivy) index from common crawl warc.wet files

## Build
Install rust (e.g. via [rustup](https://rustup.rs)).
```
make
```

## Run
```
Usage:
  warc_parser  <index> <warc_dir>
```
Where <index> is the directory of an empty index you created e.g. tantivy-cli
and <warc_dir> the path to the directory with the common crawl warc.wet files.
Depending on your system this might take a few days or weeks.
```
./target/release/tantivy_warc_indexer ../common_crawl_tantivy_index ../wet
```

Best
Andreas
