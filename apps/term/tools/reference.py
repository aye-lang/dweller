#!/usr/bin/env python3
"""The screen text of a recorded stream according to pyte (pip install
pyte), to check a parser golden against another emulator:

    reference.py FILE.bytes
"""
import sys

import pyte



class Screen(pyte.Screen):
    def select_graphic_rendition(self, *attrs, private=False):
        if not private:
            super().select_graphic_rendition(*attrs)


screen = Screen(80, 24)
stream = pyte.ByteStream(screen)
with open(sys.argv[1], "rb") as f:
    stream.feed(f.read())
for line in screen.display:
    print(line.rstrip())
print(f"cursor {screen.cursor.y},{screen.cursor.x}")
