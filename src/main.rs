use epub_parser::{epub::Book, error::EpubResult};

fn main() -> EpubResult<()> {
    let book = Book::new("book.epub")?;
    dbg!(&book.metadata);
    dbg!(&book.spine);
    let res = book.resource(&book.spine[0].path)?;
    dbg!(res);
    dbg!(res.content_str()?);
    let doc = roxmltree::Document::parse(
        "<package xmlns:dc=\"http://purl.org/dc/elements/1.1/\"><dc:title>Title</dc:title></package>",
    )?;
    dbg!(
        doc.root_element()
            .first_child()
            .unwrap()
            .tag_name()
            .namespace()
    );

    Ok(())
}
