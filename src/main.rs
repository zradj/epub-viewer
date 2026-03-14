use std::{error::Error, fs::File};

use epub_viewer::epub::EpubBook;

fn main() -> Result<(), Box<dyn Error>> {
    let book_file = File::open("book.epub")?;
    let book = EpubBook::new(book_file)?;

    println!("{book:?}");

    Ok(())
}
