
class BasicDrawer:
    def draw(img):
        img.show()

from lib.waveshare_epd import epd5in83bc as waveshare
import time

class EinkDrawer:
    def draw(img):
        epd = waveshare.EPD()
        epd.init()
        epd.Clear()
        time.sleep(1)

