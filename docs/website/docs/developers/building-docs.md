# Building the Documentation

The user guide is an MkDocs Material site. Its Python dependencies are pinned
in `docs/website/requirements.txt`.

## Create an environment

From the repository root:

```console
python3 -m venv docs/website/venv
source docs/website/venv/bin/activate
python -m pip install -r docs/website/requirements.txt
```

The virtual environment is local build output and should not be committed.

## Preview the site

```console
mkdocs serve --config-file docs/website/mkdocs.yml
```

Open `http://127.0.0.1:8000/`. MkDocs watches the Markdown and configuration
files and reloads the site after changes.

## Validate a production build

Use strict mode so broken navigation and links reported by MkDocs fail the
build:

```console
mkdocs build --strict --config-file docs/website/mkdocs.yml
```

The generated site is written to `docs/website/site/`.

## Build the Rust API reference

Rustdoc is separate from the user guide:

```console
cargo doc --workspace --no-deps --all-features
```

Open `target/doc/topohedral_tracing/index.html` to inspect it locally. Examples
in the user guide are not compiled by MkDocs, so run the crate tests after
changing code samples or public behavior.
