# tantivy_warc_indexer

tantivy_warc_indexer builds a [tantivy](https://github.com/tantivy-search/tantivy) index from common crawl warc.wet files

## Build
Install rust (e.g. via [rustup](https://rustup.rs)).
```
make
```
## Usage
```
./target/release/tantivy_warc_indexer --help
WARC Indexer

Usage:
  warc_parser [-t <threads>] [--from <from>] [--to <to>] <index> <warc_dir>
  warc_parser (-h | --help)

Options:
  -h --help      Show this help
  -t <threads>   number of threads to use, default 4
  --from <from>  skip files until from
  --to <to>      skip files after to
```

## Run

Where <index> is the directory of an empty index you created e.g. tantivy-cli
and <warc_dir> the path to the directory with the common crawl warc.wet or warc.wet.gz files.
Depending on your system this might take a few days or weeks.
```
./target/release/tantivy_warc_indexer ../common_crawl_tantivy_index ../wet
```
To create an index:
```
mkdir ../common_crawl_tantivy_index
cp template/meta.json ../common_crawl_tantivy_index/
```

Best
Andreas
