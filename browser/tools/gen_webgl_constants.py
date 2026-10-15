#!/usr/bin/env python3
"""Writes the WebGL 2 constant table of browser/engine/webgl_prelude.aye
(docs/webgl.md §3) from the pinned ANGLE's GLES3/gl3.h.

WebGL 2 has the constants of OpenGL ES 3.0 except the ones below (the
WebGL 2 specification, "Differences Between WebGL and OpenGL ES 3.0", and
the queries WebGL answers itself), plus its own `*_WEBGL` constants.

    python3 browser/tools/gen_webgl_constants.py
"""

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]
HEADER = ROOT / "browser/gpu/include/GLES3/gl3.h"
PRELUDE = ROOT / "browser/engine/webgl_prelude.aye"

# Not in WebGL 2: header macros, booleans, fixed point, binary shaders and
# programs, buffer mapping, name lengths, extension and format counts,
# texture swizzles, primitive restart, color attachments past 15, ETC2/EAC.
EXCLUDED = {
    "ES_VERSION_2_0", "ES_VERSION_3_0", "GLES_PROTOTYPES", "FALSE", "TRUE",
    "FIXED", "SHADER_COMPILER", "SHADER_BINARY_FORMATS",
    "NUM_SHADER_BINARY_FORMATS", "PROGRAM_BINARY_FORMATS",
    "NUM_PROGRAM_BINARY_FORMATS", "PROGRAM_BINARY_LENGTH",
    "PROGRAM_BINARY_RETRIEVABLE_HINT", "BUFFER_ACCESS_FLAGS", "BUFFER_MAPPED",
    "BUFFER_MAP_LENGTH", "BUFFER_MAP_OFFSET", "BUFFER_MAP_POINTER",
    "MAP_READ_BIT", "MAP_WRITE_BIT", "MAP_INVALIDATE_RANGE_BIT",
    "MAP_INVALIDATE_BUFFER_BIT", "MAP_FLUSH_EXPLICIT_BIT",
    "MAP_UNSYNCHRONIZED_BIT", "ACTIVE_ATTRIBUTE_MAX_LENGTH",
    "ACTIVE_UNIFORM_MAX_LENGTH", "ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH",
    "TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH", "INFO_LOG_LENGTH",
    "SHADER_SOURCE_LENGTH", "UNIFORM_NAME_LENGTH", "UNIFORM_BLOCK_NAME_LENGTH",
    "EXTENSIONS", "NUM_EXTENSIONS", "MAJOR_VERSION", "MINOR_VERSION",
    "NUM_COMPRESSED_TEXTURE_FORMATS", "NUM_SAMPLE_COUNTS",
    "PRIMITIVE_RESTART_FIXED_INDEX", "FRAMEBUFFER_UNDEFINED",
    "TEXTURE_SWIZZLE_R", "TEXTURE_SWIZZLE_G", "TEXTURE_SWIZZLE_B",
    "TEXTURE_SWIZZLE_A", "GREEN", "BLUE",
    "COMPRESSED_R11_EAC", "COMPRESSED_SIGNED_R11_EAC", "COMPRESSED_RG11_EAC",
    "COMPRESSED_SIGNED_RG11_EAC", "COMPRESSED_RGB8_ETC2",
    "COMPRESSED_SRGB8_ETC2", "COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2",
    "COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2", "COMPRESSED_RGBA8_ETC2_EAC",
    "COMPRESSED_SRGB8_ALPHA8_ETC2_EAC",
} | {f"COLOR_ATTACHMENT{i}" for i in range(16, 32)}

WEBGL = {
    "UNPACK_FLIP_Y_WEBGL": 0x9240,
    "UNPACK_PREMULTIPLY_ALPHA_WEBGL": 0x9241,
    "CONTEXT_LOST_WEBGL": 0x9242,
    "UNPACK_COLORSPACE_CONVERSION_WEBGL": 0x9243,
    "BROWSER_DEFAULT_WEBGL": 0x9244,
    "MAX_CLIENT_WAIT_TIMEOUT_WEBGL": 0x9247,
}


def table():
    names = {}
    for m in re.finditer(r"#define GL_(\w+)\s+(0x[0-9A-Fa-f]+|\d+)u?\b", HEADER.read_text()):
        if m.group(1) not in EXCLUDED:
            names[m.group(1)] = int(m.group(2), 0)
    names.update(WEBGL)
    return " ".join(f"{k} {v:x}" for k, v in names.items())


def main():
    src = PRELUDE.read_text()
    line = re.compile(r"^    \\\\const TABLE = '.*';$", re.M)
    assert line.search(src), "no TABLE line in webgl_prelude.aye"
    new = "    \\\\const TABLE = '" + table() + "';"
    PRELUDE.write_text(line.sub(lambda _: new, src, count=1))


if __name__ == "__main__":
    main()
