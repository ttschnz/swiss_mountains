from source.terrain_viz import generate_img

# Niesen:
# West North 2'613'742.33, 1'168'713.47
# East South 2'620'431.12, 1'161'331.14
niesen = (
    1161331, # south
    1168713, # north
    2613742,  # west
    2620431, # east
)

generate_img(*niesen, 'Niesen', step=10, offline=True)
