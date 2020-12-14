from enum import Enum
from datetime import datetime


class LogLevel(Enum):
    INFO = 1
    ERROR = 2

    @classmethod
    def max_width(cls):
        return len(sorted([enum.name for enum in cls], key=len, reverse=True)[0])

class LogCategory(Enum):
    SPOTIFY = 1
    DITHERING = 2
    EINK = 3
    ALBUMART = 4

    @classmethod
    def max_width(cls):
        return len(sorted([enum.name for enum in cls], key=len, reverse=True)[0])

def log(level, category, message):
    level_width = LogLevel.max_width()
    category_width = LogCategory.max_width()

    now = datetime.now()
    date = now.strftime("%Y-%m-%d")
    timestamp = now.strftime("%H:%M:%S")
    print(f"[{level.name.ljust(level_width)}][{category.name.ljust(category_width)}][{date}][{timestamp}] -- {message}")
    