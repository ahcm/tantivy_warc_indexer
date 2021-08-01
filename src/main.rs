#![feature(associated_type_bounds)]
use std;
use std::io;
use std::fs::File;
use std::io::BufRead;
use std::collections::HashMap;
use std::path::PathBuf;
use std::ffi::OsStr;

use docopt::Docopt;
extern crate tantivy;
use tantivy::Document;
use tantivy::Index;
use tantivy::IndexWriter;
use std::io::Read;
use flate2::read::MultiGzDecoder;


const USAGE: &'static str = "
WARC Indexer

Usage:
  warc_parser [-t <threads>] [--from <from>] [--to <to>] <index> <warc_dir>
  warc_parser (-h | --help)

Options:
  -h --help      Show this help
  -t <threads>   number of threads to use, default 4
  --from <from>  skip files until from
  --to <to>      skip files after to
";

#[derive(Debug)]
struct Args {
    arg_index: Vec<String>,
    arg_warc_dir: Vec<String>,
    flag_threads: usize
}

#[derive(Debug)]
enum WARCType
{
    WarcInfo,
    Response,
    Resource,
    Request,
    Metadata,
    Revisit,
    Conversion,
    Continuation
}

struct WARCRecord
{
    warc_version: String,
    warc_type: WARCType,
    content_length: usize,
    payload : Vec<u8>,
    header : HashMap<String, String>,
//        WARC-Type: warcinfo
//        WARC-Date: 2020-04-10T14:25:56Z
//        WARC-Filename: CC-MAIN-20200328074047-20200328104047-00001.warc.wet.gz
//        WARC-Record-ID: <urn:uuid:9f7dbec8-10d4-4829-9a28-6edcf87d4b5a>
//        Content-Type: application/warc-fields

}

fn read_record(reader: &mut dyn io::BufRead) ->  Result<Option<WARCRecord>, std::io::Error>
{
    let mut header: HashMap<String, String> = HashMap::new();
    for line in reader.lines()
    {
        let line = line?;
        let kv: Vec<_> = line.splitn(2, ':').map(|s| s.trim()).collect();
        match kv.as_slice()
        {
            [ k, v ] => header.insert(k.to_string(), v.to_string()),
            [ "WARC/1.0" ] => header.insert("WARC".to_string(),"1.0".to_string()),
            [ "" ]         => // end of header     
                {
                    let warc_version   = header.get("WARC").expect("Header Field WARC version missing.").clone();
                    let warc_type      = match header.get("WARC-Type").expect("Header Field WARC-Type missing").as_ref()
                    {
                          "conversion" => WARCType::Conversion,
                            "warcinfo" => WARCType::WarcInfo,
                        "continuation" => WARCType::Continuation,
                             "revisit" => WARCType::Revisit,
                            "metadata" => WARCType::Metadata,
                            "request"  => WARCType::Request,
                            "resource" => WARCType::Resource,
                            "response" => WARCType::Response,
                                 other => panic!("Not a known WARCType: {}", other),
                    };
                    let content_length = header
                                         .get("Content-Length").expect("Header Field Content-Length missing")
                                         .parse::<usize>().expect("number for content-length");
                    let mut payload = Vec::new();
                    {
                       let mut payload_reader = reader.take(content_length as u64);
                       let bytes_read = payload_reader.read_to_end(&mut payload)?;
                       if bytes_read != content_length
                       {
                           panic!("bytes_read != content_length")
                       }
                    }

                    // skip the record separator;
                    reader.lines().next();
                    reader.lines().next();

                    return Ok(Some(WARCRecord{warc_version,
                                              warc_type,
                                              content_length,
                                              payload,
                                              header}));
                }
            [ _, _, .. ]   => {eprintln!("ignoring unexpected multiple values");  break}, 
            [ _ ]          => {eprintln!("ignoring unexpected single value");  break},
            [  ]           => {eprintln!("ignoring unexpected empty line");  break},
        };
    }

    return Ok(None);

}


fn extract_records_and_add_to_index(index: &Index, index_writer : &IndexWriter, reader : &mut dyn BufRead) -> io::Result<()>
{
    let schema = index.schema();
    let schema_uri   = schema.get_field("uri").unwrap();
    let schema_title = schema.get_field("title").unwrap();
    let schema_body  = schema.get_field("body").unwrap();
    let schema_date  = schema.get_field("date").unwrap();

    let mut count = 0;
    while let Some(record) = read_record(reader)?
    {
        match record.warc_type
        {
            WARCType::WarcInfo =>
            {
                //eprintln!("{}", String::from_utf8(record.payload).expect("warcinfo in UTF-8"));
            },
            WARCType::Conversion =>
            {
                count += 1;
                if count % 1000 == 0 { eprint!("."); }

                let body = std::str::from_utf8(&record.payload).expect("convert to utf8 failed");
                let mut doc = Document::default();
                doc.add_text(schema_uri, &record.header.get("WARC-Target-URI").expect("get uri"));
                doc.add_text(schema_date, &record.header.get("WARC-Date").expect("get date"));
                doc.add_text(schema_body, body);
                doc.add_text(schema_title, body.lines().next().expect("title"));
                index_writer.add_document(doc);
            }
            _ => ()
        }
    }

    println!("\nTotal Records of WARC file processed: {}", count);
    Ok(())
}

fn main() -> Result<(), std::io::Error>
{
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(std::env::args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());

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
                    extract_records_and_add_to_index(&index,
                                                     &index_writer,
                                                     &mut io::BufReader::with_capacity(PER_THREAD_BUF_SIZE, MultiGzDecoder::new(file) )
                                                    )?;
                }
                else if extension == OsStr::new("wet")
                {
                    extract_records_and_add_to_index(&index,
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
