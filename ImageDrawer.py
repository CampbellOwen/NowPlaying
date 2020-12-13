
class BasicDrawer:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_value, traceback):
        pass
    def draw(self, img, red):
        img.show()
        # red.show()


from sys import platform
if not platform == "win32":
    from lib.waveshare_epd import epd5in83bc as waveshare
from PIL import Image
from sys import argv

class EinkDrawer:
    def __enter__(self):
        print("Initializing display")
        self.epd = waveshare.EPD()
        self.epd.init()
        self.epd.Clear()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        print("Going to sleep")
        self.epd.sleep()
        self.epd.Dev_exit()

    def draw(self, bw, red):
        one_bit_bw = bw.convert("1", dither=Image.NONE)
        one_bit_red = red.convert("1", dither=Image.NONE)

        epd = waveshare.EPD()

        print("Drawing to display")
        epd.display(epd.getbuffer(one_bit_bw), epd.getbuffer(one_bit_red))


if __name__ == "__main__":
    with EinkDrawer() as drawer:
        with Image.open(argv[1]) as img:
            drawer.draw(img, img)