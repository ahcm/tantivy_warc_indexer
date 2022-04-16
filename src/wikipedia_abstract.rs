use std;
use std::io;
use std::io::BufRead;

extern crate tantivy;
use tantivy::Document;
use tantivy::Index;
use tantivy::IndexWriter;

use serde;
use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Feed
{
    doc : Vec<DocEntry>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DocEntry
{
    r#abstract : String,
    //links : String,
    title : String,
    url : String,
}

pub fn extract_records_and_add_to_index(index: &Index, index_writer : &IndexWriter, reader : &mut dyn BufRead) -> io::Result<()>
{
    let schema = index.schema();
    let schema_uri      = schema.get_field("uri").unwrap();
    let schema_title    = schema.get_field("title").unwrap();
    let schema_body     = schema.get_field("body").unwrap();

    let mut src = String::new();
    reader.read_to_string(&mut src).expect("read file");
    let feed : Feed = from_str(&src).unwrap();

    let mut count = 0;
    for doc_entry in feed.doc
    {
        count += 1;
        if count % 1000 == 0 { eprint!("."); }

        let mut doc = Document::default();
        doc.add_text(schema_title, doc_entry.title);
        doc.add_text(schema_body, doc_entry.r#abstract);
        doc.add_text(schema_uri, doc_entry.url);
        index_writer.add_document(doc);
    }
    println!("\nTotal Records of WARC file processed: {}", count);
    Ok(())
}
