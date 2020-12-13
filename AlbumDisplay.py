from PIL import Image, ImageFont, ImageDraw

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
                with Image.new('RGB', (img_target_width, img_target_height), color="white") as bg:
                    bg.paste(i)

                    base_height = int(3 * (bg.height) / 4)
                    caption_width = bg.width - i.width  - 40
                    final_image = bg.copy()

        return final_image