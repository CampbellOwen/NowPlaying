import spotifyApi as S
import pprint
import os
import os.path
import subprocess
import unicodedata
import re
import time

import io

from sys import platform
from sys import argv

from AlbumDisplay import BasicInterface
from ImageDrawer import BasicDrawer, EinkDrawer
from Log import LogLevel, LogCategory, log

from PIL import Image

import secrets

def get_file_name(name, image_path):
    name = unicodedata.normalize('NFKD', name)
    name = re.sub('[^\w\s-]', '', name).strip().lower()
    name = re.sub('[-\s]+', '-', name)

    album_file_name = f'{image_path}/{name}.png'

    return album_file_name


def download_image(url, file_path):
    code, response = api.make_request(url)
    if code == 200:
        with Image.open(io.BytesIO(response.content)) as img:
            height = 428
            album_height = current_song['album_image_height']
            scale = height / album_height
            img = img.resize((int(album_height * scale), int(album_height * scale)))
            img.save(file_path)
            log(LogLevel.INFO, LogCategory.ALBUMART, f"{file_path} downloaded")

    return True


def get_dithered_album(current_song):
    image_name = f'{current_song["artist"]} - {current_song["album"]}'
    image_file_name = get_file_name(image_name, image_path)

    if not os.path.exists(image_file_name):
        download_result = download_image(current_song['album_url'], image_file_name)

        if not download_result:
            log(LogLevel.ERROR, LogCategory.ALBUMART, "Error downloading image")
            exit(1)

        log(LogLevel.INFO, LogCategory.DITHERING, "Dithering album art")
        dither_return_code = subprocess.call([dither_path, image_file_name, image_file_name])
        if not dither_return_code == 0:
            log(LogLevel.ERROR, LogCategory.DITHERING, "Dithering failed")
            exit(1)

    else:
        log(LogLevel.INFO, LogCategory.ALBUMART, f"{image_file_name} found in cache")

    return image_file_name


if len(argv) < 3 :
    print("Provide arguments for dither command and image folder")
    exit(1)

dither_path = argv[1]
image_path = argv[2]

api = S.Spotify(secrets.client_id, secrets.client_secret, secrets.refresh_token)

current_song = None
sleep_time = 5

interface_generator = BasicInterface()

with BasicDrawer() if platform == "win32" else EinkDrawer() as drawer: 
    while True:
        log(LogLevel.INFO, LogCategory.SPOTIFY, "Refreshing current song")
        new_song = api.current_song()
        if new_song is None or current_song == new_song:
            current_song = new_song
            time.sleep(sleep_time)
            continue
        current_song = new_song
        log(LogLevel.INFO, LogCategory.SONG, f"{current_song['song']} - {current_song['album']} - {current_song['artist']}")

        # Download album art 
        if current_song is None:
            exit(1)

        image_name = get_dithered_album(current_song)

        bw, red = interface_generator.create(image_name, current_song)

        drawer.draw(bw, red)
        
        time.sleep(sleep_time)