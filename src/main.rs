use epub_viewer::{epub::EpubBook, error::EpubResult};

fn main() -> EpubResult<()> {
    let book = EpubBook::new("book.epub")?;
    dbg!(&book.metadata);
    dbg!(&book.spine);
    let res = book.get_resource(&book.spine[0].clone())?;
    dbg!(res);

    Ok(())
}
