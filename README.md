# orinfar

a text editor for witches

# Principles

## Minimal

- A TUI text editor
- Simple but powerful and opinionated
- Not configurable
- Similar to base vi, but with better defaults (lsp support, etc)
- Similar in princible to [Helix](https://helix-editor.com/), but more minimal

## Reliable

- Ideally, all software should work exactly as documented and intented
    - No implementation quirks
    - No undocumented but "obvious" or "intuitive" behavior
- Not much else to say at the moment

# Definitions
- "current character": The character which the cursor is on. When in normal mode, it is character which the solid cursor block appears over.
- "word": Words are collections of characters delimited on either end by any non-alphanumeric character, unless the words begins with a non-alphanumeric non-whitespace character, in which case it is delimited by any alphanumeric or whitespace character.
    - eg: "abc123" is one word; "abc123$#!" is two words


# Actions
Actions can be broadly separated into four categories:

## Commands
Commands are single or multi-character actions that do not wait for a motion to execute. In most cases, they immedianty execute, although some do wait for additional input.
- `i`: The insert command. Enters the editor into Insert mode. Analogous to the [i command in vi]()
- `a`: The append command. Enters the editor into Insert mode and moves the cursor forward one character. Analogous to the [a command in vi]()
- `r`: The replace command. Waits for another character input as an argument, then replaces the current character with the argument character. Analogous to the [r command in vi]()
- `x`: The cut command. Deletes and copies the current character. Unlike the `d` operator, it will not remove newline characters ('\n'). If at the end of the line, it will move the cursor back one character after deleting. Analogous to the [x command in vi]()
- `p`: The paste command. Pastes the contents of the current yank register into the buffer after the current character. Analogous to the [p command in vi]()
- `o`: The newline below command. Appends a newline character ('\n') to the end of the current line, the moves the cursor to the start of the emptpy new line below. Analogous to the [o command in vi]()
- `O`: The newline above command. Appends a newline character ('\n') to the end of the previous line, the moves the cursor to the start of the empty new line above. Analogous to the [O command in vi]()

## Operators
Operators are single character actions that, once pressed in Normal mode, wait for a motion to activate. When typed in Visual mode, they immediantly activate, using the highlighted visual section as their range instead of the result of a motion.
- `y`: The yank operator. Copies every character traversed by the given motion into the current yank register.This operator will copy the current character for inclusive motions but not for exclusive motions. Analogous to the [y operator in vi]()
- `d`: The delete operator. Deletes and copies every character traversed by the given motion into the current yank register. This operator will delete the current character for inclusive motions but not for exclusive motions. Analogous to the [d operator in vi]()
- `c`: The change operator. Deletes and copies every character traversed by the given motion into the current yank register, then enters insert mode. This operator will delete and copy the current character for inclusive motions but not for exclusive motions. Analogous to the [c operator in vi]()

## Motion
Motions are single or multi-character actions that move the cursor over the buffer in some way. They can either literally move the cursor or simply "outline" some region that an operator can be applied over. They are are necessary for operators to work and are thus always chained to them, although they can be used independently. 
Some motions are inclusive, while others are exclusive. For exclusive motions, the operator will not apply to the last character in the selection (the one which the cursor lands on), while for inclusive motions, the operator will be applied to the last character. This distinction only matters when the motion is being chained to an operator.
The following descriptions of motions only describe the aforementioned independent case, but the dependent case can be inferred.
- `w`: The word motion. Moves the current cursor to the beginning of the next word. Exclusive. Analogous to the [w motion in vi]().
- `e`: The end of word motion. Moves the current cursor to the next end of a word. Inclusive. Analogous to the [e motion in vi]().
- `b`: The back word motion. Moves the current cursor backwards to the previous beginning of a word. Analogous to the [b motion in vi]().
- `_`: The beginning of line motion. Moves the current cursor backwards to first beginning of a word on the current line. In otherwords, to the first non-whitespace character in the line. Ultra-inclusive Analogous to the [_ motion in vi]().
- `$`: The end of line motion. Moves the current cursor forwards to the end of the current line, usually a newline character. Inclusive. Analogous to the [$ motion in vi]().
- `f`: The find motion. Waits for another character input, then moves the cursor forwards to the next instance of that character. If a newline character is encountered before the argument character, the motion will be aborted and not move the cursor. Inclusive. Analogous to the [f motion in vi]().

# Implementation

# Roadmap

I'll specify and revise this list when I get around to specific things and I have a better understanding of where I want to take this project

- [x] Basic Vi commands (w, b, o, d, etc)
- [ ] Additionally Vi features (a robust register system, marks, etc)
- [ ] Lsp support, syntax highlighting, etc

# Known Bugs
Please check the [stable](https://github.com/azaleacolburn/orinfar/tree/stable)  for the latest bug-free commit
