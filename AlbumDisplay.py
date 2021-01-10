from PIL import Image, ImageFont, ImageDraw, BdfFontFile

from fontTools.ttLib import TTFont

def has_glyph(font, glyph):
    for table in font['cmap'].tables:
        if ord(glyph) in table.cmap.keys():
            return True

def font_supports_text(font, text):
    f = TTFont(font)
    return all([has_glyph(f, glyph) for glyph in text])


def cut_text(max_width, font, text):
    (width, height) = font.getsize(text)

    if not width > max_width:
        return text
    
    width += (font.getsize("...")[0])
    percent_over = max_width  / width

    pos_in_string = int(len(text) * percent_over)
    if text[pos_in_string - 1] == " ":
        pos_in_string -= 1

    return text[:pos_in_string] + "..."

def line_wrap(max_width, font, text):
    (width, height) = font.getsize(text)
    if not width > max_width:
        return text
        
    words = text.split(" ")

    final_string = ""
    curr_line = ""
    for i, word in enumerate(words):
        (word_width, word_height) = font.getsize(" " + word)
        (curr_line_width, line_height) = font.getsize(curr_line)
        
        if (curr_line_width + word_width) >= max_width or word_width >= max_width:
            if word_width >= max_width:
                word = cut_text(max_width, font, word)
            final_string += curr_line + "\n" + word + "\n"
            curr_line = ""
        else:
            curr_line += word + " "

    final_string += curr_line
    return final_string

def resize_image(image, target_height):
    scale = target_height / image.height

    #log(LogLevel.INFO, LogCategory.ALBUMART, "Resizing image")
    return image.resize((int(image.width * scale), int(image.height * scale)))


class MirroredInterface:
    def __init__(self, dither_function, img_width, img_height):
        self.dither_function = dither_function

        self.artist_font = ImageFont.truetype('fonts/Consolas.ttf', size=25)
        self.album_font = ImageFont.truetype('fonts/Consolas.ttf', size=25)
        self.song_font = ImageFont.truetype('fonts/ChicagoFLF.ttf', size=70)
        self.song_font_smaller = ImageFont.truetype('fonts/ChicagoFLF.ttf', size=40)

        self.artist_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=25)
        self.album_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=25)
        self.song_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=60)
        self.song_font_smaller_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=40)

        self.img_width = img_width
        self.img_height = img_height
        self.album_height = img_height


    def create(self, album_img, song_info):
        bw_image = None
        red_image = None

        padding = 15

        with Image.open(album_img) as album, Image.new('RGB', (self.img_width, self.img_height), color="white") as bw, Image.new('RGB', (self.img_width, self.img_height), color="white") as red:
            album = resize_image(album, self.album_height)

            album_x = int((bw.width / 2 ) - (album.width / 2))
            bw.paste(album, (album_x, 0))

            album_flipped = album.transpose(method=Image.FLIP_LEFT_RIGHT)
            bw.paste(album_flipped, (album_x - album.width, 0))
            
            album_x = album_x + album.width
            bw.paste(album_flipped, (album_x, 0))
            bw = self.dither_function(bw)

            bw_draw = ImageDraw.Draw(bw)
            red_draw = ImageDraw.Draw(red)

            red_bar_middle = int(3 * (bw.height / 5))

            red_bar_height = int(bw.height / 6)
            red_bar_rectangle = [(0, red_bar_middle - (red_bar_height // 2)), (bw.width, red_bar_middle + (red_bar_height // 2))]
            red_draw.rectangle(red_bar_rectangle, fill=0)
            bw_draw.rectangle(red_bar_rectangle, fill=255)

            # Song title

            allowed_width = bw.width - (2 * padding)
            song_font = self.song_font
            useJpFont = not font_supports_text('fonts/ChicagoFLF.ttf', song_info['song'])
            if useJpFont:
                song_font = self.song_font_jp

            song_text = cut_text(allowed_width, song_font, song_info['song'])

            if not song_text == song_info['song']:
                print(song_text)
                song_text = cut_text(allowed_width, self.song_font_smaller, song_info['song'])
                song_font = self.song_font_smaller_jp if useJpFont else self.song_font_smaller 

            song_size = song_font.getsize(song_text)
            
            song_x = int((red.width / 2) - (song_size[0] / 2))
            song_y = int(red_bar_middle - (red_bar_height / 2) - (song_size[1] / 2))
            song_pos = (song_x, song_y)


            shadow_offset = 5

            bw_draw.text((song_pos[0] - shadow_offset, song_pos[1] + shadow_offset), song_text, font=song_font, fill=0)
            bw_draw.text((song_pos[0] + shadow_offset, song_pos[1] - shadow_offset), song_text, font=song_font, fill=255)
            red_draw.text((song_pos[0] - shadow_offset, song_pos[1] + shadow_offset), song_text, font=song_font, fill=(255,255,255,255))
            red_draw.text((song_pos[0] + shadow_offset, song_pos[1] - shadow_offset), song_text, font=song_font, fill=(255,255,255,255))
            
            red_draw.text(song_pos, song_text, font=song_font, fill=0)
            bw_draw.text(song_pos, song_text, font=song_font, fill=255)

            red.show()


            bw_image = bw.copy()
            red_image = red.copy()

        return bw_image, red_image


class BasicInterface:
    def __init__(self, dither_function, img_width, img_height):
        self.dither_function = dither_function

        self.artist_font = ImageFont.truetype('fonts/Consolas.ttf', size=25)
        self.album_font = ImageFont.truetype('fonts/Consolas.ttf', size=25)
        self.song_font = ImageFont.truetype('fonts/ChicagoFLF.ttf', size=67)
        self.song_font_smaller = ImageFont.truetype('fonts/ChicagoFLF.ttf', size=40)

        self.artist_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=25)
        self.album_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=25)
        self.song_font_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=67)
        self.song_font_smaller_jp = ImageFont.truetype('fonts/KosugiMaru.ttf', size=40)

        self.img_width = img_width
        self.img_height = img_height
        self.album_height = img_height - 52

    def create(self, album_img, song_info):
        caption_text = f'{song_info["song"]}\n{song_info["album"]}\n{song_info["artist"]}'

        # Create display image
        bw_image = None
        red_image = None
        with Image.open(album_img) as img, Image.new('RGB', (self.img_width, self.img_height), color="white") as bg, Image.new('RGB', (self.img_width, self.img_height), color="white") as red:
            img = resize_image(img, self.album_height)
            img = self.dither_function(img)
            bg.paste(img)

            bw_draw = ImageDraw.Draw(bg)
            red_draw = ImageDraw.Draw(red)

            # Draw Album 

            album_font = self.album_font
            if not font_supports_text('fonts/Consolas.ttf', song_info['album']):
                album_font = self.album_font_jp

            album_text_size = album_font.getsize(song_info['album'])

            year = song_info['release_date'][:4]
            year_text_size = album_font.getsize(year)
            
            album_padding = 15 
            album_max_width = self.img_width - (album_padding * 2) - (year_text_size[0] + album_padding)
            album_text = cut_text(album_max_width, album_font, song_info['album'])

            album_y = self.album_height + ((self.img_height - self.album_height) / 2)
            album_pos = (15, int(album_y - (album_text_size[1] / 2)))

            bw_draw.text(album_pos, album_text, font=album_font, fill=(0,0,0,255))

            # Draw Year
            year_pos = (self.img_width - 15 - year_text_size[0], album_pos[1])
            bw_draw.text(year_pos, year, font=album_font, fill=(0,0,0,255))

            # Draw Song Title
            max_title_width = int(0.9 * self.img_width)
            song_font = self.song_font
            useJpFont = not font_supports_text('fonts/ChicagoFLF.ttf', song_info['song'])
            if useJpFont:
                song_font = self.song_font_jp

            song_text = cut_text(max_title_width, song_font, song_info['song'])


            if not song_text == song_info['song']:
                song_text = cut_text(max_title_width, self.song_font_smaller, song_info['song'])
                song_font = self.song_font_smaller_jp if useJpFont else self.song_font_smaller 

            song_size = song_font.getsize(song_text)
            song_pos = (self.img_width - album_padding - song_size[0], int((self.img_height / 2) - (song_size[1] / 2)))

            song_shadow_offset = 3
            song_shadow_pos = (song_pos[0] + song_shadow_offset, song_pos[1] - song_shadow_offset)

            song_rect_size = (song_size[0] + (2 * album_padding), int((song_size[1] / 2) + (album_padding / 2)) )
            song_rect_pos = (self.img_width - song_rect_size[0], int(self.img_height / 2))

            song_rect_box = [song_rect_pos, (song_rect_pos[0] + song_rect_size[0] + 1, song_rect_pos[1] + song_rect_size[1] + 1)]

            red_draw.rectangle(song_rect_box, fill=(0, 0, 0, 255))
            bw_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            bw_draw.text(song_pos, song_text, font=song_font, fill=(0,0,0,255))

            # To cut out the text in the red rect
            red_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            red_draw.text(song_pos, song_text, font=song_font, fill=(255,255,255,255))

            # Draw Artist
            artist_font = self.artist_font
            if not font_supports_text('fonts/Consolas.ttf', song_info['artist']):
                artist_font = self.artist_font_jp

            artist_pos = (img.width + album_padding, int(song_rect_box[1][1] + album_padding))
            artist_max_width = self.img_width - img.width - (2 * album_padding)
            artist_text = line_wrap(artist_max_width, artist_font, song_info['artist'])
            bw_draw.multiline_text(artist_pos, artist_text, font=artist_font, fill=(0,0,0,255), align="left", spacing=10)

            # Draw track number
            track_number = song_info['track_number']
            num_tracks = song_info['total_tracks']

            dot_size = 3
            dot_spacing = 7
            all_dots_height =  (dot_size * num_tracks) + ((num_tracks - 1) * dot_spacing)

            available_height = song_pos[1]

            dot_pos = (img.width + album_padding, album_padding)
            for i in range(num_tracks):                    
                fill = (255, 255, 255, 255)
                if i+1 == track_number:
                    fill = (0, 0, 0, 255)
                
                if dot_pos[1] >= available_height:
                    dot_pos = (dot_pos[0] + dot_size + dot_spacing, album_padding)
                dot_rect = [dot_pos, (dot_pos[0] + dot_size, dot_pos[1] + dot_size)]
                bw_draw.ellipse(dot_rect, fill=fill, outline=(0,0,0,255))

                dot_pos = (dot_pos[0], dot_pos[1] + dot_size + dot_spacing)

            bw_image = bg.copy()
            red_image = red.copy()

        return bw_image, red_image