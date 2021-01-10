
class BasicDrawer:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_value, traceback):
        pass
    def draw(self, img, red):
        img.show()
        red.show()
        return
    def clear(self):
        return


from sys import argv, platform
if not platform == "win32":
    from lib.waveshare_epd import epd5in83b_V2 as waveshare
from PIL import Image
from Log import log, LogCategory, LogLevel

class EinkDrawer:
    def __enter__(self):
        log(LogLevel.INFO, LogCategory.EINK, "Initializing display")
        self.epd = waveshare.EPD()
        self.epd.init()
        self.epd.Clear()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        log(LogLevel.INFO, LogCategory.EINK, "Going to sleep")
        self.epd.sleep()
        self.epd.Dev_exit()

    def draw(self, bw, red):
        one_bit_bw = bw.convert("1", dither=Image.NONE)
        one_bit_red = red.convert("1", dither=Image.NONE)

        log(LogLevel.INFO, LogCategory.EINK, "Drawing to display")
        self.epd.display(self.epd.getbuffer(one_bit_bw), self.epd.getbuffer(one_bit_red))

    def clear(self):
        self.epd.Clear()


if __name__ == "__main__":
    with EinkDrawer() as drawer:
        with Image.open(argv[1]) as img:
            drawer.draw(img, img)