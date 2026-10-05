use entrez_rs::parser::esearch::ESearchResult;
use entrez_rs::parser::pubmed::PubmedArticleSet;

#[test]
fn esearch_deserializes_ids_and_escaped_text() {
    let result = ESearchResult::read(
        r#"<eSearchResult>
            <Count>2</Count><RetMax>2</RetMax><RetStart>0</RetStart>
            <IdList><Id>123</Id><Id>456</Id></IdList>
            <QueryTranslation>heart &amp; lung</QueryTranslation>
        </eSearchResult>"#,
    )
    .unwrap();

    assert_eq!(result.count, 2);
    assert_eq!(result.ret_max, 2);
    assert_eq!(result.id_list.ids, ["123", "456"]);
    assert_eq!(result.query_translation, "heart & lung");
}

#[test]
fn pubmed_preserves_nested_abstract_markup() {
    let result = PubmedArticleSet::read(
        r#"<PubmedArticleSet><PubmedArticle><MedlineCitation><Article>
            <ArticleTitle>Example</ArticleTitle>
            <Journal><Title>Test journal</Title></Journal>
            <Abstract><AbstractText>Before <i>outer <b>inner</b></i> after.</AbstractText></Abstract>
        </Article></MedlineCitation></PubmedArticle></PubmedArticleSet>"#,
    )
    .unwrap();

    assert_eq!(result.articles.len(), 1);
    let article = result
        .articles
        .into_iter()
        .next()
        .unwrap()
        .medline_citation
        .unwrap()
        .article
        .unwrap();
    assert_eq!(article.title.as_deref(), Some("Example"));
    assert_eq!(article.journal.unwrap().title.as_deref(), Some("Test journal"));
    assert_eq!(
        article.abstract_text.unwrap().text[0].value.as_deref(),
        Some("Before <i>outer <b>inner</b></i> after.")
    );
}
