Orinfar is a modern, minimal text editor for witches. It's largely based on the [Vi](https://www.man7.org/linux/man-pages/man1/vi.1p.html) text editor.

# Modes
Orinfar is a modal editor, much like Vi.

## Normal
The default mode. When entering text in Normal mode, instead of being written to the buffer, the text is interpreted as a stream of Actions.

This mode can be entered by pressing the `esc` key from any other mode.

## Insert
Text entered in Insert mode is written to the buffer, much like one would normally expect when typing in a text editor.

This mode can be entered with the `a`, `i`, or `o` commands.

## Visual
> [!WARNING]
> Currently, Visual mode is not implemented, nor are any related features.

While in this mode, the cursor will outline a linear highligheted area. When operators are applied while in this mode, they are applied to the entire highlighted region, as if the entire net motion of the cursor while in Visual mode were the chained motion (see below).

This mode can be entered with the `v` command.

## Meta
In Meta mode, the user is given access to a one-line text buffer in the bottom of the screen (which has no internal modes, and is essentially always in "insert" mode), where they can type in series of meta-commands.
The point of meta-commands is to either perform large scale actions on the buffer (such as find and replace), or performing io with the outside system (such as writing to a file)

Some meta-commands are a single letter, while some are multiple. Some can be followed by other meta-commands in the buffer without whitespace delimiting them, in which case they will be executed sequentially. Others cannot be followed because they take an additional argument.
Once a sequence of meta-commands are entered, they can be executed by pressing the `enter` key, which will also return the editor to Normal mode.

The meta-commands are as follows:
- `w`: The write meta-command. Writes the current contents of the buffer to the underlying file associated with the buffer. In the case that the buffer is not attatched to a path, an error will be displayed in the status-line and this commmatand will be aborted.
- `q`: The quite meta-command. Quites from the editor without writing, aborting to process.
- `o[file_path]`: The open operator. Attatches the buffer to the file at the argument path. Because it has an argument, no other meta-commands may follow it.
- `s[search]/[substitute]`: The substitute operator. Searches to current buffer for the given `[search]` string, then replaces each instance with the `[substitute]` string.

This command can be entered by pressing `:`.

# Actions
Actions are Actions can be broadly separated into three categories:

## Commands
Commands are single or multi-character actions that do not wait for a motion to execute. In most cases, they immedianty execute, although some do wait for additional input.
- `i`: The insert command. Enters the editor into Insert mode. Analogous to the `i` command in Vi.
- `a`: The append command. Enters the editor into Insert mode and moves the cursor forward one character. Analogous to the `a` command in Vi.
- `r[character]`: The replace command. Waits for another character input as an argument, then replaces the current character with the argument character. Analogous to the `r` command in Vi.
- `x`: The cut command. Deletes and copies the current character. Unlike the `d` operator, it will not remove newline characters ('\n'). If at the end of the line, it will move the cursor back one character after deleting. Analogous to the `x` command in Vi.
- `p`: The paste command. Pastes the contents of the current yank register into the buffer after the current character. Analogous to the `p` command in Vi.
- `o`: The newline below command. Appends a newline character ('\n') to the end of the current line, the moves the cursor to the start of the empty new line below. In addition, it appends spaces to the new line such that the first non-whitespace column of the new line is the same as the first non-whitespace column of the old line. Analogous to the `o` command in Vi.
- `O`: The newline above command. Appends a newline character ('\n') to the end of the previous line, the moves the cursor to the start of the empty new line above. In addition, it appends spaces to the new line such that the first non-whitespace column of the new line is the same as the first non-whitespace column of the old line. Analogous to the `O` command in Vi.
- `G`: The last row command. Moves the cursor to the first column of last row of the current buffer.
- `gg`: The first row command. Moves the cursor to the first column of first row of the current buffer.
- `u`: The undo command. Undoes the last action performed by the user. All character sequentially and continuously inserted to the buffer in Insert mode in any given "insertion session" are considered to be a single action. For example, typing `testing` in Insert mode would be a single action. All operator-motion chains are considered a single action. The cursor movement is not considered an action and thus cannot be undone, although the cursor may be moved in the process of undoing actions. At the moment, Orinfar makes no guarantees about cursor placement after this command is run, except that the cursor will remain in a valid position in the buffer.

## Operators
Operators are single character actions that, once pressed in Normal mode, wait for a motion to activate. When typed in Visual mode, they immediantly activate, using the highlighted Visual section as their range instead of the result of a motion.
- `y[motion]`: The yank operator. Copies every character traversed by the given motion into the current yank register.This operator will copy the current character for inclusive motions but not for exclusive motions. Analogous to the `y` operator in Vi.
- `d[motion]`: The delete operator. Deletes and copies every character traversed by the given motion into the current yank register. This operator will delete the current character for inclusive motions but not for exclusive motions. Analogous to the `d` operator in Vi.
- `c[motion]`: The change operator. Deletes and copies every character traversed by the given motion into the current yank register, then enters insert mode. This operator will delete and copy the current character for inclusive motions but not for exclusive motions. Analogous to the `c` operator in Vi.
- `t[motion]`: The change until operator. Deletes and copies every character traversed by the given motion, except the last character, into the current yank register, then enters insert mode. This operator will delete and copy the current character for inclusive motions but not for exclusive motions. Analogous to the `t` operator in Vi.

## Motion
Motions are single or multi-character actions that move the cursor over the buffer in some way. They can either literally move the cursor or simply "outline" some region that an operator can be applied over. They are are necessary for operators to work and are thus always chained to them, although they can be used independently. 
Some motions are inclusive, while others are exclusive. For exclusive motions, the operator will not apply to the last character in the selection (the one which the cursor lands on), while for inclusive motions, the operator will be applied to the last character. This distinction only matters when the motion is being chained to an operator.
The following descriptions of motions only describe the aforementioned independent case, but the dependent case can be inferred.
- `h`: The left motion. Moves the current cursor one column to the left. Exclusive. Analogous to the `h` motion in Vi.
- `j`: The down motion. Moves the current cursor one row down. Exclusive. Analogous to the `j` motion in Vi.
- `k`: The up motion. Moves the current cursor one row up. Exclusive. Analogous to the `k` motion in Vi.
- `l`: The right motion. Moves the current cursor one column to the right. Exclusive. Analogous to the `l` motion in Vi.
- `w`: The word motion. Moves the current cursor to the beginning of the next word. Exclusive. Analogous to the `w` motion in Vi.
- `e`: The end of word motion. Moves the current cursor to the next end of a word. Inclusive. Analogous to the `e` motion in Vi.
- `b`: The back word motion. Moves the current cursor backwards to the previous beginning of a word. Analogous to the `b` motion in Vi.
- `$`: The end of line motion. Moves the current cursor forwards to the end of the current line, usually a newline character. Inclusive. Analogous to the `$` motion in Vi.
- `_`: The beginning of line motion. Moves the current cursor backwards to first beginning of a word on the current line. In otherwords, to the first non-whitespace character in the line. Ultra-inclusive Analogous to the `_` motion in Vi.
- `f[character]`: The find motion. Waits for another character input, then moves the cursor forwards to the next instance of that character. If a newline character is encountered before the argument character, the motion will be aborted and not move the cursor. Inclusive. Analogous to the `f` motion in Vi.

# Non-actions
## Normal Mode
- `:`: Enters Meta mode. 
- `esc`: Clears the current chain of characters and sets the current count to 1. For example pressing `d`, `esc`, and then `d` will not delete the current line. Subsequently pressing `d` will delete the current line.

## Insert Mode
- `esc`: Enters Normal mode.
- `enter`: Inserts a newline character to the current cursor position. In addition to this, it inserts spaces to the new line after the newline character but before the text pulled from the old line to the new line, such that the first non-whitespace column of the new line is the same as the first non-whitespace column of the old line.
- `backspace`: Deletes the current character, moving back the cursor accordingly. If the deleted character is a space character (` `), then in addition to deleting it, subsequent (backwards) space characters will be deleted to align the number of spaces to 4 spaces. If a multiple of 4 spaces were present initially, 4 spaces will be deleted. For example, pressing delete when the following texts are before the cursor will lead to the following results: `hello world    ` (4 spaces)=> `hello world`; `hello world     ` (5 spaces) => `hello world    `. Where the arrows represent the backspace transformation.
- `tab`: Inserts 4 spaces at the current cursor position, incrementing the cursor accordingly.
- `[c]`: (Any [character](https://doc.rust-lang.org/nightly/std/primitive.char.html)) Inserts that `[c]` into the buffer at the current cursor position, incrementing the cursor accordingly.

## Meta Mode
- `[c]`: (Any [character](https://doc.rust-lang.org/nightly/std/primitive.char.html)) Inserts that `[c]` into the status line buffer at the current status line cursor position, incrementing the cursor accordingly.
- `esc`: Enters Normal mode and clears the status bar buffer.
- `enter`: Executes each command in the status line character by character, breaking as soom an a conclusive command is run. Then enters Normal mode and clears the status bar buffer.
- `backspace`: Deletes the current character, moving back the cursor accordingly.
- `left`: Moves the status line cursor left one character.
- `right`: Moves the status line cursor right one character.

## Normal, Insert
- `left`: Moves the cursor left one character (or column). Does not move on to the end of the previous line or change the cursor's row at all.
- `right`: Moves the cursor right one character (or column). Does not move on to the beginning of the next line or change the cursor's row at all.
- `up`: Moves the cursor up one rw, keeping the column the same, unless the new current line is shorter, in which case it moves to the last column.
- `down`: Moves the cursor down one row, keeping the column the same, unless the new current line is shorter, in which case it moves to the last column.

# Definitions
- "current character": The character which the cursor is on. When in normal mode, it is character which the solid cursor block appears over.
- "word": Words are collections of characters delimited on either end by any non-alphanumeric character, unless the words begins with a non-alphanumeric non-whitespace character, in which case it is delimited by any alphanumeric or whitespace character.
    - eg: "abc123" is one word; "abc123$#!" is two words
- "attatched": If the buffer is attatched to a file path, when the write meta-command i executed, the buffer will be written to that file. Also, when the load meta-command is executed, the contents of the file is loaded into the buffer.
