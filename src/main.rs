//#![feature(associated_type_bounds)]
use std;
use std::io;
use std::fs::File;
use std::path::PathBuf;
use std::ffi::OsStr;

use docopt::Docopt;
extern crate tantivy;
use tantivy::Index;
use flate2::read::MultiGzDecoder;

mod warc;
mod pubmed;
mod wikipedia_abstract;


const USAGE: &'static str = "
WARC Indexer

Usage:
  warc_parser [-t <threads>] [--from <from>] [--to <to>] -s <format> <index> <warc_dir>
  warc_parser (-h | --help)

Options:
  -h --help      Show this help
  -s <source>    type of source files (WARC or ENTREZ or WIKIPEDIA_ABSTRACT)
  -t <threads>   number of threads to use, default 4
  --from <from>  skip files until from
  --to <to>      skip files after to
";

#[derive(Debug)]
enum SourceType
{
    WARC,
    WIKIPEDIA_ABSTRACT,
    ENTREZ
}

#[derive(Debug)]
struct Args
{
    arg_index: Vec<String>,
    arg_warc_dir: Vec<String>,
    flag_threads: usize,
    flag_source: SourceType
}


fn main() -> Result<(), std::io::Error>
{
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(std::env::args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());

    let source_type = args.get_str("-s");
    let index_dir = args.get_str("<index>");
    let warc_dir  = args.get_str("<warc_dir>");
    let threads   = args.get_str("-t");
    let from      = args.get_str("--from").parse::<usize>().unwrap_or(0);
    let to        = args.get_str("--to").parse::<usize>().unwrap_or(usize::MAX);
    let nthreads : usize = threads.parse().unwrap_or(4);
    const PER_THREAD_BUF_SIZE : usize = 600 * 1024 * 1024;

    println!("Only indexing files: {} - {}", from, to);
    println!("Index dir: {:?}", index_dir);
    println!("Warc dir: {:?}", warc_dir);
    println!("Threads: {:?}", nthreads);
    println!("");

    let index_directory = PathBuf::from(index_dir);
    let index = Index::open_in_dir(&index_directory).expect("Tantivy Index Directory open failed");
    let mut index_writer = index.writer_with_num_threads(nthreads, nthreads * 4095 * 1024 * 1024).expect("index writer failed");

    let mut numfiles = 0;
    for path in std::fs::read_dir(warc_dir).unwrap()
    {
        numfiles += 1;
        if numfiles < from || numfiles > to
        {
            continue
        }
        let filename = path.unwrap().path();
        let file = File::open(&filename).unwrap();
        
        eprintln!("{}\t{}", numfiles, filename.to_string_lossy());
        match filename.extension()
        {
            Some(extension) =>
                if extension == OsStr::new("gz")
                {
                    println!("gzipped {}", source_type);
                    match source_type
                    {
                        "WARC" =>
                            warc::extract_records_and_add_to_index(&index,
                                                     &index_writer,
                                                     &mut io::BufReader::with_capacity(PER_THREAD_BUF_SIZE, MultiGzDecoder::new(file) )
                                                    )?,
                        "WIKIPEDIA_ABSTRACT" =>
                            wikipedia_abstract::extract_records_and_add_to_index(&index,
                                                     &index_writer,
                                                     &mut io::BufReader::with_capacity(PER_THREAD_BUF_SIZE, MultiGzDecoder::new(file) )
                                                    )?,
                        "ENTREZ" =>
                            pubmed::extract_records_and_add_to_index(&index,
                                                     &index_writer,
                                                     &mut io::BufReader::with_capacity(PER_THREAD_BUF_SIZE, MultiGzDecoder::new(file) )
                                                     )?,
                            _ => eprintln!("Unknown source type {}", source_type)
                    }
                }
                else if extension == OsStr::new("wet")
                {
                    warc::extract_records_and_add_to_index(&index,
                                                     &index_writer,
                                                     &mut io::BufReader::with_capacity(PER_THREAD_BUF_SIZE, file)
                                                    )?;
                }
                else
                {
                    eprintln!("Skip file, neither wet nor gz");
                },
            None => 
                    eprintln!("Skip file, neither wet nor gz"),

        }
    }
    index_writer.commit().expect("commit");
    index_writer.wait_merging_threads().expect("merging");

    Ok(())
}
