# Prospectus Graphicus — common tasks

default:
    @just --list

sync:
    uv sync

install:
    uv tool install .

run *ARGS:
    uv run prospectus {{ARGS}}

check:
    uv run python -m compileall prospectus_graphicus

test:
    uv run python -m unittest discover -s tests_py

ci: check test
