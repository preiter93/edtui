# To run this script, install vhs and run `vhs examples/app.tape`
Output "resources/app.gif"
Set Margin 10
Set Padding 2
Set BorderRadius 10
Set Width 1200
Set Height 700
Set PlaybackSpeed 0.5

Hide
Type "cargo run --example app"
Set TypingSpeed 100ms
Enter
Sleep 1.5s

# First line
Type "v$"
Show
Sleep 3.5s
Escape
Type "0jj"
Sleep 0.2s

# Second line
Type "wllllllllllllll"
Sleep 0.2s
Type "hhhhhhhhhhhhhh"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Third line
Type "wwwwwwwwwwbbbbbbbbbb"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Fourth line
Type "lllllllvlll"
Sleep 0.2s
Escape
Type "wwwwwwwwcw"
Sleep 0.2s
Type "d"
Sleep 0.2s
Type "j0j"
Sleep 0.2s

# Fifth line
Type "$"
Sleep 0.5s
Type "p"
Sleep 0.5s
Type "j0j"
Sleep 1.0s

# Sixth line
Type "/"
Sleep 0.2s
Type "c"
Sleep 0.2s
Type "o"
Sleep 1.0s
Escape
Type "j0j"
Sleep 0.2s

# Seventh line
Type "vj$"
Sleep 3.5s
Escape
Sleep 0.5s
