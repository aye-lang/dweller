#!/usr/bin/env python3
"""Writes the hand-made streams of the parser tests into tests/:
colors.bytes (SGR, 256 and true colors, the title, wide characters, line
drawing) and modes.bytes (cursor movement, erasing, insert and delete,
the scroll region, the alternate screen, reports, scrollback)."""
import os

E = "\x1b"
out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "tests")

colors = ""
for i in range(8):
    colors += f"{E}[3{i}m fg{i} {E}[0m{E}[4{i}m bg{i} {E}[0m{E}[9{i}m hi{i} {E}[0m{E}[10{i}m hb{i} {E}[m\r\n"
colors += f"{E}[1mbold{E}[0m {E}[3mitalic{E}[0m {E}[4munder{E}[0m {E}[7minverse{E}[0m {E}[9mstrike{E}[0m {E}[2mdim{E}[0m\r\n"
colors += f"{E}[1;4;31mboth{E}[22mnot bold{E}[24;39mplain{E}[0m\r\n"
colors += f"{E}[38;5;208morange256{E}[0m {E}[48;5;57mbg256{E}[0m {E}[38;2;10;200;30mtruecolor{E}[0m {E}[48:2::1:2:3mcolon{E}[0m {E}[38:5:99mcolon256{E}[0m\r\n"
colors += f"{E}]0;the title\x07wide: 中文 é 😀 end\r\n"
colors += f"{E}(0lqqk\r\nx  x\r\nmqqj{E}(B plain q\r\n"
colors += f"{E}]2;second title{E}\\{E}P ignored dcs {E}\\after strings\r\n"
with open(os.path.join(out, "colors.bytes"), "wb") as f:
    f.write(colors.encode())

modes = "".join(f"line {i}\r\n" for i in range(1, 31))
modes += f"{E}[H{E}[2Jtop left{E}[5;10Hrow5 col10{E}[3Cxx{E}[2Dy{E}[2Bdown{E}[Aup"
modes += f"{E}[10;1Habcdefghij{E}[10;4H{E}[2P{E}[10;2H{E}[3@{E}[10;8H{E}[1X"
modes += f"{E}[12;1Herase me please{E}[12;7H{E}[K{E}[13;1Hkeep right side{E}[13;5H{E}[1K"
modes += f"{E}[15;20r{E}[15;1Hr15\r\nr16\r\nr17\r\nr18\r\nr19\r\nr20\r\nr21 scrolled region\r\nr22"
modes += f"{E}[16;1H{E}[L inserted{E}[18;1H{E}[M{E}[15;1H{E}M{E}Mreverse index"
modes += f"{E}[r{E}[22;1Hwrap:" + "w" * 80 + "!"
modes += f"{E}[?7l{E}[24;70Hno wrap past the edge{E}[?7h"
modes += f"{E}7{E}[?1049h{E}[2;2Halternate screen{E}[6n{E}[c{E}[?1049l{E}8"
modes += f"{E}[1;40H{E}[4hins{E}[4l{E}[2;40Hab{E}[3bc\tTAB\x08\x08B"
modes += f"{E}[?25l{E}[?1h{E}[>c{E}[99;99H"
with open(os.path.join(out, "modes.bytes"), "wb") as f:
    f.write(modes.encode())
