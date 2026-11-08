#!/bin/sh
# Plants a crash for the fuzz workflow's `plant` input (never committed to
# the shipped source): the HTML target reads past its input when the input
# contains `<noembed>`, a dictionary token that no seed file contains, so
# only a mutant finds it.
#
#     sh .github/scripts/fuzz-plant.sh
set -eu
cd "$(dirname "$0")/../.."
python3 - <<'PY'
from pathlib import Path
p = Path("browser/fuzz/html-target.aye")
head = "fn fuzzHtml(alloc: Allocator, input: []u8) AllocError!void {\n"
plant = '    if text.indexOf(input, "<noembed>") != null {\n        @println(input[input.len..input.len + 1])\n    }\n\n'
s = p.read_text()
assert head in s
p.write_text(s.replace(head, head + plant))
PY
echo "fuzz-plant: browser/fuzz/html-target.aye"
