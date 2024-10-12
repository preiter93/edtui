# To run this script, install vhs and run `vhs examples/app.tape`
Output "resources/app.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set FontSize 32
Set Width 2400
Set Height 1000
Set PlaybackSpeed 0.4

Hide
Type "cargo run --example app"
Set TypingSpeed 100ms
Enter
Sleep 1.5s

# Introduction
Type "v$"
Show
Sleep 3.5s
Escape
Type "0jj"
Sleep 0.2s

# Simple navigation
Type "wllllllllllllll"
Sleep 0.2s
Type "hhhhhhhhhhhhhh"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Word forward
Type "wwwwwwwwwwbbbbbbbbbb"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Select between
Type "lllllllvlll"
Sleep 0.2s
Escape
Type "wwwwwwwwsiw"
Sleep 0.2s
Type "d"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Copy paste
Type "$"
Sleep 0.5s
Type "p"
Sleep 1.5s
Type "j0j"
Sleep 0.5s

# Search
Type "/"
Sleep 0.2s
Type "c"
Sleep 0.2s
Type "o"
Sleep 1.5s
Escape
Type "j0j"
Sleep 0.2s

# Syntax highlighting
Sleep 1.5s
Type "jjwww"
Type@0.5s "%%%%%"
Sleep 1.0s

# Contribution
Type "jjvj$"
Sleep 4.5s
Escape
Sleep 0.5s
