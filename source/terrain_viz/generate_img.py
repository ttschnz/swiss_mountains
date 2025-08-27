import numpy as np
from scipy.spatial import cKDTree

import matplotlib.pyplot as plt
# from matplotlib.colors import LightSource
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
from matplotlib.tri import Triangulation
import matplotlib.animation as animation

from utils import url_to_ref
from source import swissalti3d, swissimage

PLOT_PATH = "./plots"

examples = {
    # Niesen:
    # West North 2'613'742.33, 1'168'713.47
    # East South 2'620'431.12, 1'161'331.14
    "niesen": (
        1161331, # south
        1168713, # north
        2613742,  # west
        2620431, # east
    )
}

def generate_img(south,north,west,east, name, step=100, offline=False, dpi=300):
    bounding_box = [south, north, west, east]
    if not offline:
        # cache altitude points
        swissalti3d.cache.initialize_cache()
        url_list = swissalti3d.fetch.get_url_list((bounding_box[0],bounding_box[1]),(bounding_box[2],bounding_box[3]))
        for url in url_list:
            print(f"caching {url_to_ref(url)}")
            swissalti3d.fetch.fetch_and_extract(url)
        quit()
        # cache colors
        img_url_list = swissimage.fetch.get_url_list((bounding_box[0],bounding_box[1]),(bounding_box[2],bounding_box[3]))
        for url in img_url_list:
            print(f"caching {url_to_ref(url)}")
            swissimage.fetch.fetch_and_extract(url)

    print("fetching altitude points from cache")
    # get points inside range
    data = swissalti3d.cache.get_many_from_cache_filtered(step=step, minx=bounding_box[2], maxx=bounding_box[3], miny=bounding_box[0],maxy=bounding_box[1])
    if data is None:
        data = []

    print("fetching colors from cache")
    # get colors inside range
    imgdata = swissimage.cache.get_many_from_cache_filtered(step=step*100, minx=bounding_box[2], maxx=bounding_box[3], miny=bounding_box[0],maxy=bounding_box[1])
    if imgdata is None:
        imgdata = []

    print("processing colors")
    imgdata_map = {}
    for (x,y,r,g,b) in imgdata:
        imgdata_map[(x,y)] = (r,g,b)

    x,y,z = zip(*data)


    xs = np.array(x)
    ys = np.array(y)
    zs = np.array(z)

    print("generating triangles")
    tri = Triangulation(xs, ys)

    img_points = np.array(list(imgdata_map.keys()))   # shape (N, 2)
    img_colors = np.array(list(imgdata_map.values())) / 255.0  # normalized

    tree = cKDTree(img_points)

    print("coloring triangles")
    colors = []

    for (x, y) in zip(xs, ys):
        if (x, y) in imgdata_map:
            rgb = np.array(imgdata_map[(x, y)]) / 255.0
        else:
            # fallback: find closest point
            dist, idx = tree.query([x, y])
            rgb = img_colors[idx]
        colors.append(rgb)
    colors = np.array(colors)

    # build polygons + average colors
    polys = []
    face_colors = []
    for tri_indices in tri.triangles:
        verts = [(xs[i], ys[i], zs[i]) for i in tri_indices]
        polys.append(verts)
        # average color of triangle vertices
        face_colors.append(colors[tri_indices].mean(axis=0))

    print("creating plot")
    fig = plt.figure(figsize=(8, 6))
    ax = fig.add_subplot(111, projection="3d")

    mesh = Poly3DCollection(polys, facecolors=face_colors, linewidths=0.2, alpha=1.0)
    ax.add_collection3d(mesh)

    ax.set_xlabel("X")
    ax.set_ylabel("Y")
    ax.set_zlabel("Z")

    ax.set_box_aspect((1, 1, 1/2))

    ax.axis('off')

    # function to rotate
    def rotate(angle):
        ax.view_init(elev=30, azim=angle)

    print("animating rotation")
    # create animation
    ani = animation.FuncAnimation(fig, rotate, frames=range(0, 360), interval=50)

    # save gif
    ani.save(f"{PLOT_PATH}/{name}.gif", writer='pillow', dpi=dpi)
