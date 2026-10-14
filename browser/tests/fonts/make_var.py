# Makes tinyvar.ttf, a variable font with a `wght` axis (100 to 900,
# default 400) and one glyph, "D": a box whose stems and advance grow
# with the weight (fontTools):
#     python3 make_var.py
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib.tables.TupleVariation import TupleVariation

def ring(x0, y0, x1, y1, t):
    # A box with a hole: stems `t` thick.
    p = TTGlyphPen(None)
    p.moveTo((x0, y0)); p.lineTo((x0, y1)); p.lineTo((x1, y1)); p.lineTo((x1, y0)); p.closePath()
    p.moveTo((x0 + t, y0 + t)); p.lineTo((x1 - t, y0 + t)); p.lineTo((x1 - t, y1 - t)); p.lineTo((x0 + t, y1 - t)); p.closePath()
    return p.glyph()

fb = FontBuilder(1000, isTTF=True)
order = [".notdef", "space", "D"]
fb.setupGlyphOrder(order)
fb.setupCharacterMap({0x20: "space", 0x44: "D"})
fb.setupGlyf({".notdef": ring(50, 0, 450, 700, 50), "space": TTGlyphPen(None).glyph(), "D": ring(100, 0, 500, 700, 60)})
fb.setupHorizontalMetrics({".notdef": (500, 50), "space": (300, 0), "D": (600, 100)})
fb.setupHorizontalHeader(ascent=800, descent=-200)
fb.setupNameTable({"familyName": "TinyVar", "styleName": "Regular"})
fb.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
fb.setupPost()
fb.setupFvar([("wght", 100, 400, 900, "Weight")], [])
# At 900 the stems are 180 units thick (the hole shrinks), the box 80
# units wider and the advance (phantom point 2) 80 units more; at 100
# the stems are 20 units thick.
heavy = [(0, 0), (0, 0), (80, 0), (80, 0), (120, 120), (-40, 120), (-40, -120), (120, -120), (0, 0), (80, 0), (0, 0), (0, 0)]
light = [(0, 0), (0, 0), (0, 0), (0, 0), (-40, -40), (40, -40), (40, 40), (-40, 40), (0, 0), (0, 0), (0, 0), (0, 0)]
fb.setupGvar({"D": [TupleVariation({"wght": (0, 1, 1)}, heavy), TupleVariation({"wght": (-1, -1, 0)}, light)]})
fb.font.save("tinyvar.ttf")
print("ok")
