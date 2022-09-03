# lyra

A TUI app for viewing Logitech Media Server playback

------------------------------------------------------------------------------

_lyra_ is a lightweight, read-only "viewport" that displays the current playlist
of a selected player connected to the provided Logitech Media Server (LMS).

At the moment it doesn't really do much, but I have managed to create a generic
JSONRPC query framework that can send arbitrary commands to the LMS and attempt
to parse the returned results.

More to come in the future.

For related projects, take a look at my more feature-rich TUI controller for LMS
written in Python, [horizon](https://github.com/Nynergy/horizon).
