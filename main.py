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

from PIL import Image

import secrets

def download_image(url, name):
    image_name = unicodedata.normalize('NFKD', name)
    image_name = re.sub('[^\w\s-]', '', image_name).strip().lower()
    image_name = re.sub('[-\s]+', '-', image_name)

    album_file_name = f'{image_path}/{image_name}.png'

    if not os.path.exists(album_file_name):
        code, response = api.make_request(url)
        if code == 200:

            with Image.open(io.BytesIO(response.content)) as img:
                height = 396
                album_height = current_song['album_image_height']
                scale = height / album_height
                img = img.resize((int(album_height * scale), int(album_height * scale)))
                img.save(album_file_name)

    else:
        print(f'[{album_file_name}] --  already exists')

    return album_file_name


if len(argv) < 3 :
    print("Provide arguments for dither command and image folder")
    exit(1)

dither_path = argv[1]
image_path = argv[2]

api = S.Spotify(secrets.client_id, secrets.client_secret, secrets.refresh_token)
pp = pprint.PrettyPrinter(indent=4)

current_song = None
sleep_time = 5

interface_generator = BasicInterface()

with BasicDrawer() if platform == "win32" else EinkDrawer() as drawer: 
    while True:
        new_song = api.current_song()
        if new_song is None or current_song == new_song:
            current_song = new_song
            print(f"Song hasn't changed, sleeping for {sleep_time}s")
            time.sleep(sleep_time)
            continue
        current_song = new_song
        pp.pprint(current_song) 

        # Download album art 
        if current_song is None:
            exit(1)

        image_name = f'{current_song["artist"]} - {current_song["album"]}'
        image_name = download_image(current_song['album_url'], image_name)

        if image_name is None:
            exit(1)

        print('[INFO] -- Dithering album art')
        dither_return_code = subprocess.call([dither_path, image_name, image_name])
        if not dither_return_code == 0:
            print('[ERROR] -- Dithering failed')
            exit(1)
                

        img = interface_generator.create(image_name, current_song)

        drawer.draw(img)
        
        print(f"Sleeping for {sleep_time}s")
        time.sleep(sleep_time)