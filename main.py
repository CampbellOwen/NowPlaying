import spotifyApi as S
import pprint
import os
import os.path
import subprocess
import unicodedata
import re

from sys import platform
from sys import argv

from wand.image import Image
from wand.display import display
from wand.color import Color
from wand.drawing import Drawing
from wand.font import Font

import secrets

if len(argv) < 3 :
    print("Provide arguments for dither command and image folder")
    exit(1)

dither_path = argv[1]
image_path = argv[2]

api = S.Spotify(secrets.client_id, secrets.client_secret, secrets.refresh_token)
pp = pprint.PrettyPrinter(indent=4)

current_song = api.current_song()
pp.pprint(current_song) 

# Download album art 
if current_song is None:
    exit()

image_name = f'{current_song["artist"]} - {current_song["album"]}'
image_name = unicodedata.normalize('NFKD', image_name)
image_name = re.sub('[^\w\s-]', '', image_name).strip().lower()
image_name = re.sub('[-\s]+', '-', image_name)

album_file_name = f'{image_path}/{image_name}.jpg'
album_file_name_png = f'{image_path}/{image_name}.png'

if not os.path.exists(album_file_name_png):
    code, response = api.make_request(current_song['album_url'])
    if code == 200:
        with open(album_file_name, 'wb') as f:
            f.write(response.content)
        print(f'[{current_song["album"]}] --  image saved')

        print('[INFO] -- Converting album art to png')
        if platform == "win32":
            convert_return_code = subprocess.call(['magick', 'convert', album_file_name, album_file_name_png])
        else:
            convert_return_code = subprocess.call(['convert', album_file_name, album_file_name_png])
        if not convert_return_code == 0:
            print('[ERROR] -- Converting failed')
            exit()

        print('[INFO] -- Dithering album art')
        dither_return_code = subprocess.call([dither_path, album_file_name_png, album_file_name_png])
        if not dither_return_code == 0:
            print('[ERROR] -- Dithering failed')
            exit()
        
        print('[INFO] -- Removing JPG version')
        os.remove(album_file_name)
else:
    print(f'[{album_file_name}] --  already exists')

img_target_width = 600
img_target_height = 448

album_height = 396

caption_text = f'{current_song["song"]}\n{current_song["album"]}\n{current_song["artist"]}'

# Create display image
with Image(filename=album_file_name_png) as img:
    (width, height) = img.size
    with img.clone() as i:
        scale = album_height / height
        i.resize(int(i.width * scale), int(i.height * scale))
        with Image(width=img_target_width, height=img_target_height, background=Color(string="grey")) as bg:
            bg.composite(i, 0, 0)

            base_height = int(3 * (bg.height) / 4)
            caption_width = bg.width - i.width  - 40
            # bg.caption(
            #     caption_text, 
            #     20, 
            #     base_height, 
            #     caption_width, 
            #     int(bg.height / 4),
            #     Font('Embryonic.ttf'))

            with Drawing() as draw:

                draw.font = 'Consolas'
                draw.font_size = 21
                draw.text(20, bg.height - 17, current_song['album'])
                draw.text(bg.width - 60, bg.height - 17, current_song['release_date'][:4])
                draw.draw(bg)
                display(bg)