
from Log import log, LogCategory, LogLevel
from PIL import Image
from sys import argv, platform


class BasicDrawer:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        pass

    def sleep(self):
        pass

    def wakeup(self):
        pass

    def draw(self, img, red):
        img = img.convert('RGB')
        red = red.convert('RGB')

        red_pixels = red.load()
        black_pixels = img.load()
        for x in range(red.width):
            for y in range(red.height):
                if red_pixels[x, y][0] <= 0:
                    black_pixels[x, y] = (255, 0, 0)
        img.show()

    def clear(self):
        pass


if not platform == "win32":
    from lib.waveshare_epd import epd5in83b_V2 as waveshare


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

    def wakeup(self):
        log(LogLevel.INFO, LogCategory.EINK, "Waking up display")
        self.epd.init()

    def sleep(self):
        log(LogLevel.INFO, LogCategory.EINK, "Putting display to sleep")
        self.epd.sleep()

    def draw(self, bw, red):
        one_bit_bw = bw.convert("1", dither=Image.NONE)
        one_bit_red = red.convert("1", dither=Image.NONE)

        log(LogLevel.INFO, LogCategory.EINK, "Drawing to display")
        self.epd.display(self.epd.getbuffer(one_bit_bw),
                         self.epd.getbuffer(one_bit_red))

    def clear(self):
        log(LogLevel.INFO, LogCategory.EINK, "Clearing display")
        self.epd.Clear()


if __name__ == "__main__":
    with EinkDrawer() as drawer:
        with Image.open(argv[1]) as img:
            drawer.draw(img, img)
