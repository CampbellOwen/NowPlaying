# NowPlaying

## Installation
1. Build the dithering project with cmake (or just use the checked in `dither.exe` if you're on windows)
2. Get a `refresh_token`, `client_id`, and `client_secret` from the Spotify API and add to `secrets.py`
3. Edit `ImageDrawer.py` to import the right version of the eink library from `lib`
4. Edit `config.py` with the width/height of the eink display
5. Run `main.py <path to dither program> <image cache folder>`

One day I'll get around to making all that way easier I promise :D

![Photo of e-ink display](photos/best/20201213_111308657_iOS.jpg)
