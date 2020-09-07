import shutil
from pathlib import Path
from functools import reduce
import json

import click
import numpy as np
import matplotlib.pyplot as plt

from .utils import (
    collect_cards,
    collect_tags,
    crop,
    crop_card,
    crop_cards,
    crop_tag,
    filter_cards,
    imread,
    imsave,
    imshow,
    match,
    mse,
    rgb2gray,
)

ROOT = Path(__file__).parent.parent


@click.group()
def monsterbook():
    pass


def page_metadata():
    tab_counts = [
        ("red", 1),
        ("orange", 3),
        ("lightgreen", 4),
        ("green", 3),
        ("lightblue", 3),
        ("blue", 2),
        ("purple", 2),
        ("black", 2),
        ("gold", 2),
    ]
    # fmt:off
    page_counts = [
        13, 
        25, 25, 1,
        25, 25, 25, 10,
        25, 25, 20,
        25, 25, 1,
        25, 6,
        25, 8,
        25, 5,
        25, 25,
    ]
    # fmt:on
    meta = []
    for color, count in tab_counts:
        meta += [dict(tab_color=color, tab_index=i) for i in range(count)]
    for i, count in enumerate(page_counts):
        meta[i]["page_id"] = i
        meta[i]["card_count"] = count
    return meta


def mkdir(path, overwrite):
    if path.is_dir() and overwrite:
        shutil.rmtree(path)
    path.mkdir(parents=True)


@monsterbook.command()
@click.option("--source", default=str(ROOT / "data" / "raw" / "reference"))
@click.option("--output", default=str(ROOT / "data" / "processed" / "reference"))
@click.option("--overwrite/--no-overwrite", default=True)
def reference_book(source, output, overwrite):
    """Crop and label a set of reference images."""
    source = Path(source).resolve()
    output = Path(output).resolve()
    mkdir(output, overwrite)

    ref_paths = list(source.glob("*"))
    metadata = page_metadata()
    assert len(ref_paths) == len(metadata), "reference screenshots wrong length"

    for meta, path in zip(metadata, ref_paths):
        img = crop(imread(path))
        label = f'{meta["page_id"]:02}_{meta["tab_color"]}_{meta["tab_index"]}'
        filename = output / f"{label}.png"
        click.echo(f"writing {filename.relative_to(ROOT)}")
        imsave(filename, img)


@monsterbook.command()
@click.option("--reference", default=str(ROOT / "data" / "processed" / "reference"))
@click.option("--output", default=str(ROOT / "data" / "processed" / "empty"))
@click.option("--overwrite/--no-overwrite", default=True)
def empty_card(reference, output, overwrite):
    reference = Path(reference).resolve()
    output = Path(output).resolve()
    mkdir(output, overwrite)

    # get the last card of the first page
    img = imread(sorted(reference.glob("*"))[0])
    empty_card = crop_card(img, 4, 4)

    filename = output / "empty.png"
    click.echo(f"writing {filename.relative_to(ROOT)}")
    imsave(filename, empty_card)


@monsterbook.command()
@click.argument("source", type=click.Path(file_okay=False))
@click.option(
    "--locations",
    default="2-10,2-2,2-14,4-17,4-0",
    help="comma-separated list of [page]-[index] values for seed values",
)
@click.option("--empty", default=str(ROOT / "data" / "processed" / "empty"))
@click.option("--output", default=str(ROOT / "data" / "processed" / "seed_tags"))
@click.option("--overwrite/--no-overwrite", default=True)
@click.option("--threshold", default=4000, help="mse from the emtpy card to keep")
@click.option("--show", default=True)
def seed_tags(source, locations, empty, output, overwrite, threshold, show):
    screenshots = [rgb2gray(crop(imread(path))) for path in Path(source).glob("*")]
    assert len(screenshots) == len(
        page_metadata()
    ), f"{source} has wrong number of pages"

    empty = rgb2gray(imread(Path(empty) / "empty.png"))

    # get the set of initial tags from the card locations
    def get_tag(loc, cards, offsets):
        return crop_tag(cards[offsets[loc[0]] + loc[1]])

    cards = collect_cards(screenshots, empty)
    metadata = page_metadata()
    offsets = np.cumsum([0] + [meta["card_count"] for meta in metadata])
    parsed_locations = [
        list(map(int, entry.split("-"))) for entry in locations.split(",")
    ]
    initial_tags = [get_tag(loc, cards, offsets) for loc in parsed_locations]

    if show:
        for tag in initial_tags:
            imshow(tag)

    # create the canonical seed tag from the bitwise and of many tags
    d = {}
    diffs = []
    for tag in collect_tags(screenshots, empty):
        index, diff = match(tag, initial_tags)
        diffs += diff
        d[index] = d.get(index, []) + [tag]
    if show:
        plt.title("mse differences from the initial tags")
        plt.hist(diffs)
        plt.show()

    res = []
    for i in sorted(d.keys()):
        click.echo(f"found {len(d[i])} entries for {i}")
        res.append(reduce(np.bitwise_and, d[i]))
    seed_tags = res

    # write the results
    output = Path(output)
    mkdir(output, overwrite)
    for i, tag in enumerate(seed_tags):
        imsave(output / f"{i+1}.png", tag)


@monsterbook.command()
@click.argument("source", type=click.Path(file_okay=False))
@click.option("--reference", default=str(ROOT / "data" / "processed" / "reference"))
@click.option("--empty", default=str(ROOT / "data" / "processed" / "empty"))
def thresholds(source, reference, empty):
    """Plot thresholds against the empty page."""
    screenshots = [rgb2gray(crop(imread(path))) for path in Path(source).glob("*")]
    refs = [rgb2gray(imread(path)) for path in Path(reference).glob("*")]
    empty = rgb2gray(imread(Path(empty) / "empty.png"))

    # Check the empty card against all of the reference cards, except
    # the first page because a snail has been seen.
    cards = sum(map(crop_cards, refs[1:]), [])
    _, diffs = match(empty, cards)
    print(len(diffs))
    plt.title("mse of empty card against reference cards")
    plt.hist(diffs)
    plt.show()

    cards = filter_cards(cards, empty, threshold=500)
    _, diffs = match(empty, cards)
    plt.title("mse of empty card against reference non-empty cards")
    plt.hist(diffs)
    plt.show()

    # Check against the actual monsterbook set, filtering out the empty cards
    cards = filter_cards(sum(map(crop_cards, screenshots), []), empty, threshold=500)
    _, diffs = match(empty, cards)
    plt.title("mse of empty card against test non-empty cards")
    plt.hist(diffs, bins=20)
    plt.show()


@monsterbook.command()
@click.argument("source", type=click.Path(file_okay=False))
@click.argument("output", type=click.Path(file_okay=False))
@click.option("--reference", default=str(ROOT / "data" / "processed" / "reference"))
@click.option("--entries", default=str(ROOT / "entries.txt"))
@click.option("--seed-tags", default=str(ROOT / "data" / "processed" / "seed_tags"))
@click.option("--empty", default=str(ROOT / "data" / "processed" / "empty"))
@click.option("--overwrite/--no-overwrite", default=True)
def transcribe(source, output, reference, entries, seed_tags, empty, overwrite):
    screenshots = [
        rgb2gray(crop(imread(path))) for path in sorted(Path(source).glob("*"))
    ]
    refs = [rgb2gray(imread(path)) for path in sorted(Path(reference).glob("*"))]
    empty = rgb2gray(imread(Path(empty) / "empty.png"))
    seeds = [rgb2gray(imread(path)) for path in Path(seed_tags).glob("*")]
    metadata = page_metadata()
    offsets = np.cumsum([0] + [meta["card_count"] for meta in metadata]).tolist()
    entries = Path(entries).read_text().strip().split("\n")

    EMPTY_THRESHOLD = 500
    UNSEEN_THRESHOLD = 5000

    data = []
    # NOTE: O(n^2) convolutions
    for img in screenshots:
        index, _ = match(img, refs)
        cards = filter_cards(crop_cards(img), empty, EMPTY_THRESHOLD)
        for i, card in enumerate(cards):
            uid = offsets[index] + i
            name = entries[uid]
            count = 0
            if mse(card, empty) > UNSEEN_THRESHOLD:
                count, _ = match(crop_tag(card), seeds)
                count += 1
            data.append(dict(uid=uid, name=name, count=count))

    output = Path(output)
    mkdir(output, overwrite)
    with (output / "out.json").open("w") as fp:
        json.dump({"data": data}, fp, indent=2)


@monsterbook.command()
def book_metadata():
    print(json.dumps(page_metadata(), indent=2))
