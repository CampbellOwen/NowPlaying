from PIL import Image, ImageFont, ImageDraw, BdfFontFile

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


class BasicInterface:
    def __init__(self):
        self.artist_font = ImageFont.truetype('Consolas.ttf', size=25)
        self.album_font = ImageFont.truetype('Consolas.ttf', size=25)
        self.song_font = ImageFont.truetype('ChicagoFLF.ttf', size=67)
        self.song_font_smaller = ImageFont.truetype('ChicagoFLF.ttf', size=40)

        # with open('cherry-13-r.bdf', 'rb') as fp:
            # BdfFontFile.BdfFontFile(fp).save('cherry-13-r.pil')
        # self.album_font = ImageFont.load('cherry-13-r.pil')

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

            song_rect_box = [song_rect_pos, (song_rect_pos[0] + song_rect_size[0] + 1, song_rect_pos[1] + song_rect_size[1] + 1)]

            red_draw.rectangle(song_rect_box, fill=(0, 0, 0, 255))
            bw_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            bw_draw.text(song_pos, song_text, font=song_font, fill=(0,0,0,255))

            # To cut out the text in the red rect
            red_draw.text(song_shadow_pos, song_text, font=song_font, fill=(255,255,255,255))
            red_draw.text(song_pos, song_text, font=song_font, fill=(255,255,255,255))

            # Draw Artist

            artist_pos = (img.width + album_padding, int(song_rect_box[1][1] + album_padding))
            artist_max_width = img_target_width - img.width - (2 * album_padding)
            artist_text = line_wrap(artist_max_width, self.artist_font, song_info['artist'])
            bw_draw.multiline_text(artist_pos, artist_text, font=self.artist_font, fill=(0,0,0,255), align="left", spacing=10)
            print(f"ARTIST\n{artist_text}")


            # Draw track number

            track_number = song_info['track_number']
            num_tracks = song_info['total_tracks']

            dot_size = 3
            dot_spacing = 7
            all_dots_height =  (dot_size * num_tracks) + ((num_tracks - 1) * dot_spacing)

            available_height = song_pos[1]

            # dot_pos = (img.width + album_padding, int((available_height / 2) - (all_dots_height / 2)))
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