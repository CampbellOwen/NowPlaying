from PIL import Image, ImageFont, ImageDraw

class BasicInterface:
    def __init__(self):
        self.artist_font = ImageFont.truetype('Consolas.ttf', size=20)
        self.album_font = ImageFont.truetype('Consolas.ttf', size=21)
        self.song_font = ImageFont.truetype('ChicagoFLF.ttf', size=20)

    def create(self, album_img, song_info):
        img_target_width = 600
        img_target_height = 448

        album_height = 396

        caption_text = f'{song_info["song"]}\n{song_info["album"]}\n{song_info["artist"]}'

        # Create display image
        final_image = None
        with Image.open(album_img) as img:
            (width, height) = img.size
            with Image.new('RGB', (img_target_width, img_target_height), color="white") as bg:
                bg.paste(img)

                draw = ImageDraw.Draw(bg)

                album_text_size = self.album_font.getsize(song_info['album'])

                album_y = album_height + ((img_target_height - album_height) / 2)
                album_pos = (15, int(album_y - (album_text_size[1] / 2)))

                draw.text(album_pos, song_info['album'], font=self.album_font, fill=(0,0,0,255))

                final_image = bg.copy()

        return final_image