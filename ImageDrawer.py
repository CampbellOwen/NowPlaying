
class BasicDrawer:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_value, traceback):
        pass
    def draw(self, img):
        img.show()


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
        self.epd.Dev_exit()

    def draw(self, img):
        one_bit = img.convert("1", dither=Image.NONE)
        blank = Image.new("1", img.size, 255)

        epd = waveshare.EPD()

        print("Drawing to display")
        epd.display(epd.getbuffer(one_bit), epd.getbuffer(blank))

        print("Going to sleep")
        epd.sleep()

if __name__ == "__main__":
    with EinkDrawer() as drawer:
        with Image.open(argv[1]) as img:
            drawer.draw(img)