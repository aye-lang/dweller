# Writing modes in layout

How `writing-mode` works in `browser/layout/` (Q6.14). The layout code is
written for horizontal text: x is the inline axis, y the block axis. It
stays that way. A vertical box is laid out by the same code in its own
logical space, and the result is turned into physical coordinates.

## Spaces

Every box has a `mode` (its own writing mode: `HORIZONTAL`, `VERTICAL_LR`,
`VERTICAL_RL`; `sideways-*` count as their vertical twin) and a `space`
(the mode of the box whose content it is part of). `enterSpaces` sets
both after the box tree is built:

- inline boxes, text and line breaks take their parent's mode;
- a box whose mode is parallel to its space takes the space (a
  `vertical-lr` box in a `vertical-rl` parent flows like its parent: left);
- an absolutely positioned box is in the horizontal space: it is laid out
  after placement, in absolute physical coordinates;
- the root is in the horizontal space. `propagateWritingMode` gives the
  root box `body`'s `writing-mode` and `direction` first (the principal
  writing mode).

A box with `mode != space` is turned: it starts an orthogonal flow.

## Styles

Layout reads physical properties (`width`, `margin-top`, `left`). In a
vertical space each box reads a turned copy of its computed style
(`turnedStyle`): `width`/`height`, `min-*`, `max-*`, `overflow-x`/`-y` and
the two border spacings swapped, and the sides of margin, padding, border
width and inset permuted (`vertical-lr`: top and left swap, bottom and
right swap; `vertical-rl`: the logical top is the physical right). A
turned box has two: `style` in its parent's space (its margins and
position) and `inner` in its own (its size and content); `inside` says
which one is in `style` now. `leaveSpaces` puts the physical styles back
(`Box.physical`) after the absolutely positioned boxes are laid out, so
paint, transforms and script geometry read physical values.

A document without a vertical box (`Tree.vertical` false) takes none of
these paths; its styles are never copied.

## Layout of a turned box

`turns(t, i)` is true for a turned box that is not being laid out inside.
The entry points ask it first:

- `layoutBlockLevel`, `layoutAtomic` -> `layoutTurned`: the box is sized
  in its own space by `layoutFitted` (shrink-to-fit inline size in the
  available size: the containing block's block size when definite, else
  the viewport's; css-writing-modes-4, 7.3.1), the root by
  `layoutBlockLevel` itself (it fills the viewport). Then `turnSubtree`
  turns the geometry and the box gets its outer margins.
- `layoutContents` (flex and grid items, table cells) ->
  `layoutTurnedContents`: the width the container gave is the box's
  block size (`forced_h` in its own space).
- `intrinsic` -> `turnedIntrinsic`: both intrinsic widths are the block
  size of the fitted box.
- `layoutAbsolute` -> `layoutTurnedAbsolute`: the whole problem is turned
  (containing block, static position), solved by `layoutAbsolute`, and
  turned back; static positions are then taken physically.

A turned box is a formatting context root. Incremental layout
(`relayout`) answers false in a tree with a vertical box: the tree is
laid out from scratch.

## Turning geometry

`turnSubtree` swaps x and y of every box, line, fragment and marker under
the box (`turnedRect`), and mirrors them in their container's content box
when a block axis runs right to left (`Turn.flip_x`: the box is
`vertical-rl`; `Turn.flip_y`: its parent's space is). Margins, borders
and paddings are permuted the same way (`turnedEdges`). A nested turned
subtree is turned once per level, so each level only knows its parent's
space. `Frag.turned` is flipped each time: paint draws a turned text
fragment a quarter turn clockwise (`paint/turned-text.aye`,
`paint/draw-turned.aye`), its baseline `ascent` from the right edge.

## Not done

See the Left list of `docs/plans/cards/Q6.14.md`.
