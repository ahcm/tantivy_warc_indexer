use std;
use std::io;
use std::io::BufRead;

extern crate tantivy;
use tantivy::Document;
use tantivy::Index;
use tantivy::IndexWriter;

use entrez_rs::parser::pubmed::PubmedArticleSet;

pub fn extract_records_and_add_to_index(index: &Index, index_writer : &IndexWriter, reader : &mut dyn BufRead) -> io::Result<()>
{
    let schema = index.schema();
    //let schema_uri   = schema.get_field("uri").unwrap();
    let schema_title = schema.get_field("title").unwrap();
    let schema_body  = schema.get_field("body").unwrap();
    //let schema_date  = schema.get_field("pubdate").unwrap();
    //let schema_authors  = schema.get_field("author").unwrap();
    let schema_journal  = schema.get_field("journal").unwrap();

    let mut doc = String::new();
    reader.read_to_string(&mut doc).expect("read file");
    let pm_parsed = PubmedArticleSet::read(&doc);
    let mut count = 0;
    for pubmed_article in pm_parsed.expect("parsed").articles
    {
        count += 1;
        if count % 1000 == 0 { eprint!("."); }

        let mut doc = Document::default();
        let article = pubmed_article.medline_citation.expect("medline_citation")
                 .article.expect("article");
        if let Some(title) = article.title
        {
            doc.add_text(schema_title, title);
        }
        doc.add_text(schema_journal, article.journal.expect("journal").title.expect("journal"));
        if let Some(abstract_text) = article.abstract_text
        {
            for text in abstract_text.text
            {
                if let Some(value) = text.value
                {
                    doc.add_text(schema_body, value);
                }
            }
        }
        index_writer.add_document(doc);
    }
    println!("\nTotal Records of WARC file processed: {}", count);
    Ok(())
}
