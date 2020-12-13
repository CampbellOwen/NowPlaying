
class BasicDrawer:
    def draw(img):
        img.show()

from lib.waveshare_epd import epd5in83bc as waveshare
from PIL import Image
from sys import argv

class EinkDrawer:
    def draw(img):

        one_bit = img.convert("1", dither=Image.NONE)
        blank = Image.new("1", img.size, 255)

        print("Initializing display")
        epd = waveshare.EPD()
        epd.init()
        epd.Clear()

        print("Drawing to display")
        epd.display(epd.getbuffer(one_bit), epd.getbuffer(blank))

        print("Going to sleep")
        epd.sleep()
        epd.Dev_exit()

if __name__ == "__main__":
    with Image.open(argv[1]) as img:
        EinkDrawer.draw(img)