import numpy as np
import matplotlib.pyplot as plt
import matplotlib.image as mpimg
from scipy.signal import convolve2d

CROP_SHAPE = (225, 165)


def imread(path):
    return mpimg.imread(path)


def imshow(img):
    plt.imshow(img, cmap=plt.get_cmap("gray"))
    plt.show()


def imsave(path, img):
    mpimg.imsave(path, img)


# https://stackoverflow.com/a/12201744
def rgb2gray(rgb):
    return (np.dot(rgb[..., :3], [0.2989, 0.5870, 0.1140]) * 255).astype(int)


def crop_win(img):
    x, y = 152, 295
    shape = CROP_SHAPE
    return img[x : x + shape[0], y : y + shape[1]]


def crop_mac(img):
    x, y = 412, 569
    # twice the resolution of normal
    shape = CROP_SHAPE[0] * 2, CROP_SHAPE[1] * 2
    return img[x : x + shape[0], y : y + shape[1]]


def crop(img):
    if img.shape[:2] not in ((600, 800), (768, 1024), (768, 1366)):
        return crop_mac(img)
    else:
        return crop_win(img)


def crop_card(img, i, j):
    num_rows = 5
    num_cols = 5
    h = int(img.shape[0] / num_rows)
    w = int(img.shape[1] / num_cols)
    return img[i * h : (i + 1) * h, j * w : (j + 1) * w]


def crop_cards(img):
    num_rows = 5
    num_cols = 5
    cards = []
    for i in range(num_rows):
        for j in range(num_cols):
            card = crop_card(img, i, j)
            cards.append(card)
    return cards


def crop_tag_mac(card):
    x, y = 62, 11
    shape = 18, 12
    return card[x : x + shape[0], y : y + shape[1]]


def crop_tag_win(card):
    x, y = 31, 5
    shape = 9, 6
    return card[x : x + shape[0], y : y + shape[1]]


def crop_tag(img):
    if img.shape[:2] != (45, 33):
        return crop_mac_tag(img)
    else:
        return crop_win(img)


def sobel_filter(img):
    kernel_x = np.array([[1, 0, -1], [2, 0, -2], [1, 0, -1]])
    kernel_y = kernel_x.T
    x = convolve2d(img, kernel_x) + convolve2d(img, kernel_y)
    x[x < 0] = 0
    x[x > 0] = 255
    return x


def mse(a, b):
    """Mean Squared Error."""
    return ((a - b) ** 2).mean()


def match(img, refs, filter_func=lambda x: x):
    """Return the index into to a list of reference images
    of the closest match."""
    diffs = []
    for ref in refs:
        diff = round(mse(filter_func(img), filter_func(ref)))
        diffs.append(diff)
    index = diffs.index(min(diffs))
    return index, diffs


def filter_cards(cards, empty, threshold=500):
    return [card for card in cards if mse(card, empty) > threshold]


def collect_tags(images, empty, threshold=4000):
    """By default, collect tags where a card has been seen"""
    tags = []
    for img in images:
        for card in filter_cards(crop_cards(img), empty, threshold):
            tags.append(crop_tag(card))
    return tags


def collect_cards(screenshots, empty):
    cards = []
    for img in screenshots:
        cards += filter_cards(crop_cards(img), empty)
    return cards
