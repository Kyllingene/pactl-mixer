# pamixer
### a gui and command-line program volume mixer

requires `pactl`, usually found in `libpulse`

## cli usage:
- `pamixer [options]`
    - `-l | --list` - list all sources
    - `-i | -id <id>` - select a source by id; takes precedence over `--name`
    - `-n | --name <name>` - select a source by name (partial match, e.g. `abc` would match `abc` and `123abcdef`)
    - `-v | --volume <vol>` - set the sources volume
    - `-m | --mute` - mutes the source
    - `-u | --unmute` - unmutes the source; takes precedence over `--mute`
