from PIL import Image,ImageFont,ImageDraw

class BasicInterface:
    def create(album_img, song_info):
        img_target_width = 600
        img_target_height = 448

        album_height = 396

        caption_text = f'{song_info["song"]}\n{song_info["album"]}\n{song_info["artist"]}'

        # Create display image
        final_image = None
        with Image.open(album_img) as img:
            (width, height) = img.size
            with img.copy() as i:
                # scale = album_height / height
                # i = i.resize((int(i.width * scale), int(i.height * scale)))
                with Image.new('RGB', (img_target_width, img_target_height), color="white") as bg:
                    bg.paste(i)

                    base_height = int(3 * (bg.height) / 4)
                    caption_width = bg.width - i.width  - 40
                    # bg.caption(
                    #     caption_text, 
                    #     20, 
                    #     base_height, 
                    #     caption_width, 
                    #     int(bg.height / 4),
                    #     Font('Embryonic.ttf'))

                    # with Drawing() as draw:

                        # draw.font = 'Consolas'
                        # draw.font_size = 21
                        # draw.text(20, bg.height - 17, song_info['album'])
                        # draw.text(bg.width - 60, bg.height - 17, song_info['release_date'][:4])
                        # draw.draw(bg)
                        # display(bg)
                    final_image = bg.copy()


        # im = I.open(album_img)
        # im.show()
        return final_image