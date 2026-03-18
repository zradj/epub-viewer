use std::fs::File;

use epub_viewer::{epub::EpubBook, error::{EpubError, EpubResult}};

fn main() -> EpubResult<()> {
    let book = EpubBook::new("abc.epub")?;
    dbg!(&book.metadata);
    dbg!(&book.spine);
    let res = book.get_resource(&book.spine[0])?;
    dbg!(String::from_utf8_lossy(&res));

    Ok(())
}
