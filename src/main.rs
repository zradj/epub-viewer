use std::fs::File;

use epub_viewer::{epub::EpubBook, error::EpubError};

fn main() -> Result<(), EpubError> {
    let book_file = File::open("book.epub")?;
    let book = EpubBook::new(book_file)?;

    println!("{book:?}");

    Ok(())
}
