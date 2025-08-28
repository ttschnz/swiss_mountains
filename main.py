from source.terrain_viz import generate_img,PLOT_PATH
from source.anki_export import create_deck
from source.swissnames3d_pkt import get_peaks

if __name__ == '__main__':
    center = (1199436, 2600340)
    peak_inclusion_radius = 20000

    peaks = list(filter(lambda peak_data: (peak_data[0]-center[0])**2+(peak_data[1]-center[1])**2 < peak_inclusion_radius**2, get_peaks()))
    print(f"{len(peaks)} peaks found in the range of {peak_inclusion_radius}m")

    names = []
    paths = []

    for (north, east, peak_height, name) in peaks:
        radius = peak_height*2

        print(f"generating image for {name}")
        #generate_img(south, north, west, east, name, width=160, offline=False, static=True, animated_extension='gif')
        #generate_img((north, east), radius, name, width=160, offline=False, static=True, animated_extension='gif')
        names.append(name)
        paths.append(f"{PLOT_PATH}/{name}.gif")

    create_deck(names, paths, "Mountains")
