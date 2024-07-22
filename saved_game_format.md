This writeup specifies a simple file format for saving/loading Onitama games, I'll call it the `.oni` file format for reference.

It would be good if a single file format can support both arbitrary game states, as well as record the history of a given game for analysis, replay, or what have you. This simple specification does that, while remaining reasonably human readable. Custom cards are not currently supported, but it should be easy to extend the format later to support these. I've tried to specify the format in such a way that it is space-efficient too.

# Specification
1. All whitespace characters are ignored, not counting rule 2.
2. All text between a '`#`' character and a '`\n`' character is ignored, to allow comments.
3. The characters '`0`','`1`','`2`','`3`' and '`.`' are reserved for defining the initial board position and must not be used elsewhere.
4. If the first character is one of the reserved characters, the file defines an initial board position. Otherwise, the file only contains a move history and the initial board state is the default one.
	1. It is assumed that the *red* player begins, so this is implied. *This is unlike the real game, but gameplay-wise it is functionally the same, as red and blue colors could be swapped with an otherwise identical setup.*
	2. The initial board position is defined by 5 groups of 5 reserved characters, for a total of 25 consecutive characters, with the blue side at the 'top', and the red side at the 'bottom', see the example diagram below.
	3. '`.`' characters denote empty squares.
	4. '`0`' and '`1`' denotes red and blue disciples.
	5. '`2`' and '`3`' denotes red and blue senseis.

	   The default board would be defined as follows:
		```
		11311
		.....
		.....
		.....
		00200
		```
5. Following an optional initial board state, the first 5 characters denote the available cards in the initial board state (before any moves are made).
	1. The cards are denoted using the characters in the [[#Card identifiers]] list.
	2. The cards are defined in the following order: The two cards for red, the two cards for blue and the 'transfer' card.
6. The characters following the optional board definition record the move history.
	1. The move history is a sequence of move records.
	2. A move record is 3 consecutive characters, denoting in order the *card used*, the *starting position* and the *ending position*.
	3. The starting and ending position are denoted by the 25 characters '`a-y`', in correspondence with the 25 squares, as per the following diagram:
		```
		Blue
		-----
		abcde
		fghij
		klmno
		pqrst
		uvwxy
		-----
		Red
		```
	4. The used card is denoted using the characters in the [[#Card identifiers]] list.
	5. Capitalization is optional, but using lowercase for positions and uppercase for cards is encouraged for readability.

## Card identifiers:
B: Boar
C: Cobra
D: Dragon
E: Eel
F: Frog
G: Goose
H: Horse
M: Mantis
O: Ox
R: Rabbit
T: Tiger
Q: Crab
K: Crane
L: Elephant
X: Monkey
U: Rooster

# Example
With whitespace and comments
```
# Example spec, specifies an initial board position and
# three moves of game history

# Non-standard start, the senseis begin one step forward
# Board positions in comments for reference
11.11  #  abcde
..3..  #  fghij
.....  #  klmno
..2..  #  pqrst
00.00  #  uvwxy

# The five cards in use
BXLUT

# Moves
Brs # red sensei moves right using boar
Ehl # blue sensei moves down and left using elephant
Tvl # red disciple captures blue sensei using tiger, game over
```
Without whitespace and comments
```
11.11..3.........2..00.00BXLUTBrsEhlTvl
```