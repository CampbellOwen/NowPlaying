from PIL import Image, ImageFont, ImageDraw

def cut_text(max_width, font, text):

    (width, height) = font.getsize(text)

    if not width > max_width:
        return text
    
    width += (font.getsize("...")[0])
    percent_over = max_width  / width

    pos_in_string = int(len(text) * percent_over)

    return text[:pos_in_string] + "..."

class BasicInterface:
    def __init__(self):
        self.artist_font = ImageFont.truetype('Consolas.ttf', size=36)
        self.album_font = ImageFont.truetype('Consolas.ttf', size=21)
        self.song_font = ImageFont.truetype('ChicagoFLF.ttf', size=67)
        self.song_font_smaller = ImageFont.truetype('ChicagoFLF.ttf', size=40)

    def create(self, album_img, song_info):
        img_target_width = 600
        img_target_height = 448

        album_height = 396

        caption_text = f'{song_info["song"]}\n{song_info["album"]}\n{song_info["artist"]}'

        # Create display image
        bw_image = None
        red_image = None
        with Image.open(album_img) as img, Image.new('RGB', (img_target_width, img_target_height), color="white") as bg, Image.new('RGB', (img_target_width, img_target_height), color="white") as red:
            bg.paste(img)

            bw_draw = ImageDraw.Draw(bg)
            red_draw = ImageDraw.Draw(red)

            # Draw Album 
            album_text_size = self.album_font.getsize(song_info['album'])

            year = song_info['release_date'][:4]
            year_text_size = self.album_font.getsize(year)
            
            album_padding = 15 
            album_max_width = img_target_width - (album_padding * 2) - (year_text_size[0] + album_padding)
            album_text = cut_text(album_max_width, self.album_font, song_info['album'])

            album_y = album_height + ((img_target_height - album_height) / 2)
            album_pos = (15, int(album_y - (album_text_size[1] / 2)))

            bw_draw.text(album_pos, album_text, font=self.album_font, fill=(0,0,0,255))

            # Draw Year
            year_pos = (img_target_width - 15 - year_text_size[0], album_pos[1])
            bw_draw.text(year_pos, year, font=self.album_font, fill=(0,0,0,255))

            # Draw Song Title
            max_title_width = int(0.9 * img_target_width)
            song_font = self.song_font
            song_text = cut_text(max_title_width, self.song_font, song_info['song'])

            if not song_text == song_info['song']:
                song_text = cut_text(max_title_width, self.song_font_smaller, song_info['song'])
                song_font = self.song_font_smaller

            song_size = song_font.getsize(song_text)
            song_pos = (img_target_width - album_padding - song_size[0], int((img_target_height / 2) - (song_size[1] / 2)))

            song_shadow_offset = 3
            song_shadow_pos = (song_pos[0] + song_shadow_offset, song_pos[1] - song_shadow_offset)

            song_rect_size = (song_size[0] + (2 * album_padding), int((song_size[1] / 2) + (album_padding / 2)) )
            song_rect_pos = (img_target_width - song_rect_size[0], int(img_target_height / 2))

            red_draw.rectangle([song_rect_pos, (song_rect_pos[0] + song_rect_size[0] + 1, song_rect_pos[1] + song_rect_size[1] + 1)], fill=(0, 0, 0, 255))
            bw_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            bw_draw.text(song_pos, song_text, font=song_font, fill=(0,0,0,255))

            # To cut out the text in the red rect
            red_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            red_draw.text(song_pos, song_text, font=song_font, fill=(255,255,255,255))


            bw_image = bg.copy()
            red_image = red.copy()

        return bw_image, red_image