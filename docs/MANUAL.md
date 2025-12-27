Orinfar is a modern, minimal text editor for witches. It's largely based on the [Vi](https://www.man7.org/linux/man-pages/man1/vi.1p.html) text editor.

# Modes
Orinfar is a modal editor, much like Vi.

## Normal
The default mode. When entering text in Normal mode, instead of being written to the buffer, the text is interpreted as a stream of Actions.

This mode can be entered by pressing the `esc` key from any other mode.

## Insert
Text entered in Insert mode is written to the buffer, much like one would normally expect when typing in a text editor.

This mode can be entered with either the `a` or `i` command.

## Visual
While in this mode, the cursor will outline a linear highligheted area. When operators are applied while in this mode, they are applied to the entire highlighted region, as if the entire net motion of the cursor while in Visual mode were the chained motion (see below).

This mode can be entered with the `v` command.

## Meta
In Meta mode, the user is given access to a one-line text buffer in the bottom of the screen (which has no internal modes, and is essentially always in "insert" mode), where they can type in series of meta-commands.
The point of meta-commands is to either perform large scale actions on the buffer (such as find and replace), or performing io with the outside system (such as writing to a file)

Some meta-commands are a single letter, while some are multiple. Some can be followed by other meta-commands in the buffer without whitespace delimiting them, in which case they will be executed sequentially. Others cannot be followed because they take an additional argument.
Once a sequence of meta-commands are entered, they can be executed by pressing the `enter` key, which will also return the editor to Normal mode.

The meta-commands are as follows:
- `w`: The write meta-command. Writes the current contents of the buffer to the underlying file associated with the buffer. In the case that the buffer is not attatched to a path, an error will be displayed in the status-line and this command will be aborted.
- `q`: The quite meta-command. Quites from the editor without writing, aborting to process.
- `o[file_path]`: The open operator. Attatches the buffer to the file at the argument path. Because it has an argument, no other meta-commands may follow it.

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
- `o`: The newline below command. Appends a newline character ('\n') to the end of the current line, the moves the cursor to the start of the emptpy new line below. Analogous to the `o` command in Vi.
- `O`: The newline above command. Appends a newline character ('\n') to the end of the previous line, the moves the cursor to the start of the empty new line above. Analogous to the `O` command in Vi.

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
- `w`: The word motion. Moves the current cursor to the beginning of the next word. Exclusive. Analogous to the `w` motion in Vi.
- `e`: The end of word motion. Moves the current cursor to the next end of a word. Inclusive. Analogous to the `e` motion in Vi.
- `b`: The back word motion. Moves the current cursor backwards to the previous beginning of a word. Analogous to the `b` motion in Vi.
- `$`: The end of line motion. Moves the current cursor forwards to the end of the current line, usually a newline character. Inclusive. Analogous to the `$` motion in Vi.
- `_`: The beginning of line motion. Moves the current cursor backwards to first beginning of a word on the current line. In otherwords, to the first non-whitespace character in the line. Ultra-inclusive Analogous to the `_` motion in Vi.
- `f[character]`: The find motion. Waits for another character input, then moves the cursor forwards to the next instance of that character. If a newline character is encountered before the argument character, the motion will be aborted and not move the cursor. Inclusive. Analogous to the `f` motion in Vi.

# Definitions
- "current character": The character which the cursor is on. When in normal mode, it is character which the solid cursor block appears over.
- "word": Words are collections of characters delimited on either end by any non-alphanumeric character, unless the words begins with a non-alphanumeric non-whitespace character, in which case it is delimited by any alphanumeric or whitespace character.
    - eg: "abc123" is one word; "abc123$#!" is two words
- "attatched": If the buffer is attatched to a file path, when the write meta-command i executed, the buffer will be written to that file. Also, when the load meta-command is executed, the contents of the file is loaded into the buffer.
