from typing_extensions import List
import genanki

ANKI_PACKAGES_PATH = './anki'
# Define a unique model ID and deck ID (random large integers)
MODEL_ID = 1234567890
DECK_ID = 987654321

# Define the note model with two fields: Image and Name
model = genanki.Model(
    MODEL_ID,
    'png image to name model',
    fields=[
        {'name': 'Image'},
        {'name': 'Name'},
    ],
    templates=[
        {
            'name': 'Card 1',
            'qfmt': '{{Image}}',  # Front side shows the image
            'afmt': '{{FrontSide}}<hr><div>{{Name}}</div>',  # Back side shows the name
        },
    ])

def create_deck(names: List[str], animation_paths: List[str], package_name:str):
    # Create a deck
    deck = genanki.Deck(
        DECK_ID,
        package_name
    )

    for name, anim_path in zip(names, animation_paths):
        # Prepare the png image filename (make sure the file exists in the same folder or provide path)
        # Create a note with the image filename and the entered name
        note = genanki.Note(
            model=model,
            fields=[f"<img src='{anim_path.split('/')[-1]}' />", name]
        )
        # Add the note to the deck
        deck.add_note(note)

    # Create the package including the media file
    package = genanki.Package(deck)
    package.media_files = animation_paths  # Add the png image as media

    print("Note image fields:", [anim_path.split("/")[-1] for anim_path in animation_paths])
    print("Media files:", animation_paths)

    # Save the deck as an .apkg file which can be imported into Anki
    package.write_to_file(f'{ANKI_PACKAGES_PATH}/{package_name}.apkg')
