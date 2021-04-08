import spotifyApi as S
import pprint
import os
import os.path
import subprocess
import unicodedata
import re
import time
import random

import io

from sys import platform
from sys import argv

from AlbumDisplay import BasicInterface, MirroredInterface, RawAlbumInterface
from ImageDrawer import BasicDrawer, EinkDrawer
from Log import LogLevel, LogCategory, log

from PIL import Image

import secrets

from config import img_height, img_width

def get_file_name(name, image_path):
    name = unicodedata.normalize('NFKD', name)
    name = re.sub('[^\w\s-]', '', name).strip().lower()
    name = re.sub('[-\s]+', '-', name)

    album_file_name = f'{image_path}/{name}.png'

    return album_file_name


def download_image(api, url, file_path):
    if not os.path.exists(file_path):
        code, response = api.make_request(url)
        if code == 200:
            with Image.open(io.BytesIO(response.content)) as img:
                img.save(file_path)
                log(LogLevel.INFO, LogCategory.ALBUMART, f"{file_path} downloaded")
        else:
            return False
    else:
        log(LogLevel.INFO, LogCategory.ALBUMART, f"{file_path} found in cache")

    return True


def resize_image(image_path, target_height):
    album_height = current_song['album_image_height']
    scale = img_height / album_height
    working_image_path = f'{image_path}/curr.png'

    log(LogLevel.INFO, LogCategory.ALBUMART, "Resizing image")
    with Image.open(image_file_name) as img:
        img = img.resize((int(album_height * scale), int(album_height * scale)))
        img.save(working_image_path)


def get_dithered_album(current_song, image_path, img_height):


    log(LogLevel.INFO, LogCategory.DITHERING, "Dithering album art")
    dither_return_code = subprocess.call([dither_path, working_image_path, working_image_path])
    if not dither_return_code == 0:
        log(LogLevel.ERROR, LogCategory.DITHERING, "Dithering failed")
        exit(1)


    return working_image_path

def dither_function(dither_path, image_path):
    def dither(img):
        working_image_path = f"{image_path}/temp.png" 
        img.save(working_image_path)
        log(LogLevel.INFO, LogCategory.DITHERING, "Dithering album art")
        dither_return_code = subprocess.call([dither_path, working_image_path, working_image_path])
        if not dither_return_code == 0:
            log(LogLevel.ERROR, LogCategory.DITHERING, "Dithering failed")
            exit(1)
        return Image.open(working_image_path)


    return dither


if len(argv) < 3 :
    print("Provide arguments for dither command and image folder")
    exit(1)

dither_path = argv[1]
image_path = argv[2]

api = S.Spotify(secrets.client_id, secrets.client_secret, secrets.refresh_token)

current_song = None
sleep_active = 5
sleep_inactive = 15

dither = dither_function(dither_path, image_path)

interfaces = [BasicInterface(dither, img_width, img_height), MirroredInterface(dither, img_width, img_height), RawAlbumInterface(dither, img_width, img_height)]

with BasicDrawer() if platform == "win32" else EinkDrawer() as drawer: 
    counter = 0
    while True:
        log(LogLevel.INFO, LogCategory.SPOTIFY, "Refreshing current song")
        new_song = api.current_song()
        if new_song is None or current_song == new_song:
            if new_song is None and not current_song is None:
                drawer.wakeup()
                drawer.clear()
                drawer.sleep()
            sleep_time = sleep_inactive if new_song is None else sleep_active
            current_song = new_song
            time.sleep(sleep_time)
            continue
        current_song = new_song
        log(LogLevel.INFO, LogCategory.SONG, f"{current_song['song']} - {current_song['album']} - {current_song['artist']}")

        # Download album art 
        if current_song is None:
            exit(1)
        
        image_name = f'{current_song["artist"]} - {current_song["album"]}'
        image_file_name = get_file_name(image_name, image_path)

        download_result = download_image(api, current_song['album_url'], image_file_name)

        if not download_result:
            log(LogLevel.ERROR, LogCategory.ALBUMART, "Error downloading image")
            exit(1)

        bw, red = random.choice(interfaces).create(image_file_name, current_song)

        drawer.wakeup()
        if counter > 50:
            counter = 0
            # Hopefully this helps with the red bleeding into the black
            drawer.clear()
        drawer.draw(bw, red)
        counter += 1
        
        drawer.sleep()
        time.sleep(sleep_active)