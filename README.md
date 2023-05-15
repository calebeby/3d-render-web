- https://twisty-puzzles.netlify.app/
- https://2x2--twisty-puzzles.netlify.app/
- https://3x3--twisty-puzzles.netlify.app/
- https://megaminx--twisty-puzzles.netlify.app/
- https://dino-starminx--twisty-puzzles.netlify.app/
- https://compy-cube--twisty-puzzles.netlify.app/
- https://starminx--twisty-puzzles.netlify.app/

Ideas going forward:

- Phased solving (by piece type)
- Evaluation of "distance from solved"

  Currently:

  - Number of piece faces that are correct
  - Number of pieces that are correct

  Future:

  - Minimum number of turns for each piece to be in the correct position and oriented correctly

- Continue to integrate trie for most-similar move set to minimize metamove effects and see how it works

TODO:

All the corners can be solved and all the edges can be in the right position, but n (even) of the edges are flipped orientation

When solving the corners (ignoring the edges), all the corners can be in the right position but three of them have the wrong orientation.

To flip a pair of edges:

1. Use a 3-cycle affecting those two edges
1. Use simple turns to flip one of those three edges, not affecting the others
1. Invert the 3-cycle
1. Invert the simple turns to flip

With a given 3-cycle instead:

1. Determine and apply setup moves A
1. Apply pre-selected 3-cycle B
1. Determine and apply simple moves C
1. Invert 3-cycle B
1. Invert simple moves C
1. Invert setup moves A

In other words,

[A, B, C, B', C', A']

The 3-cycle B is pre-determined
C is a "flip algorithm" determined to match that 3-cycle,
so B, C, B', C' is a general flip algorithm

Then only A needs to be determined for any specific situation to apply the flipping to the necessary pieces.

This was checked for edges.

Now checking for corners
I think the flip-one-corner algorithm is more complicated?
