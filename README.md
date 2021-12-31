# monsterbook transcription

## quickstart

### bootstrapping

We'll generate the assets necessary for the web application by looking at two
sets of monsterbook entries.

Create the working data directory at the root of the application. Under
`data/raw/reference`, add uncropped in-game screenshots taken in order of colors
from a character that has collected few cards. There should be 22 pages. The
monsterbook that is going to be transcribed is placed under `data/raw/main`.

```bash
# set up python
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt

# command-line tools
python -m python --help

# generate the cropped reference pages
python -m python reference-book

# generate the empty page
python -m python empty-card

# generate the seed tags (the number in the lower left of the card)
python -m python seed-tags data/raw/main

# optional: check the histograms to determine thresholds
python -m python thresholds data/raw/main

# transcribe the monsterbook
python -m python transcribe data/raw/main data/processed/main
```

### svelte

This is taken from the svelte template readme.

Install the dependencies...

```bash
npm install
```

...then start [Rollup](https://rollupjs.org):

```bash
npm run dev
```

Navigate to [localhost:5000](http://localhost:5000). You should see your app
running. Edit a component file in `src`, save it, and reload the page to see
your changes.

By default, the server will only respond to requests from localhost. To allow
connections from other computers, edit the `sirv` commands in package.json to
include the option `--host 0.0.0.0`.

#### Building and running in production mode

To create an optimised version of the app:

```bash
npm run build
```

You can run the newly built app with `npm run start`.
