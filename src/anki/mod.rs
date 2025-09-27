use std::{path::{Path, PathBuf}};

use anyhow::{Error, Result};
use genanki_rs::{Deck, Field, Model, Note, Package, Template};



pub fn create_deck(gif_paths: &[PathBuf], mountain_names: &[&str], target_file: &Path)->Result<()>{
    let anki_model = Model::new(
        0598447264,
        "SwissMountains Model",
        vec![Field::new("Mountain Name"), Field::new("gif")],
        vec![Template::new("Card 1")
            .qfmt("{{gif}}")
            .afmt(r#"{{FrontSide}}<hr id="answer">{{Mountain Name}}"#)],
    );


    let file_names = gif_paths
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|os_str| os_str.to_str())
                .ok_or(Error::msg("could not find gif"))
        })
        .collect::<Result<Vec<_>>>()?;

    
    
    let notes = file_names.iter().zip(mountain_names).map(move |(path, name)|{
        let anki_model = anki_model.clone();
        Note::new(anki_model, vec![*name, &format!(r#"<img src="{path}" />"#)])
    });
    
    let mut deck = Deck::new(123456789, "Mountains", "Swiss Mountains 3D renders");
    for note in notes {
        deck.add_note(note?);
    }

    let gif_paths_str = gif_paths
        .iter()
        .map(|path| {
            path.to_str()
                .ok_or(Error::msg("could not find gif"))
        })
        .collect::<Result<Vec<_>>>()?;
    dbg!("adding gifs to package ({:?})", gif_paths.to_vec());
    let mut package = Package::new(vec![deck], gif_paths_str)?;
    package.write_to_file(target_file.to_str().ok_or(Error::msg("could not read path to anki file"))?)?;

    Ok(())
}