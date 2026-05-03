Output docs/demo.gif

Set FontSize 18
Set Width 1200
Set Height 675
Set Theme "Dracula"
Set TypingSpeed 50ms
Set PlaybackSpeed 0.8

# Launch game (binary already built)
Type "./target/debug/slay-tui --debug"
Enter
Sleep 3s

# Map screen — enter floor 1 combat
Enter
Sleep 2s

# Combat: play Strike (card 1)
Type "1"
Enter
Sleep 1s

# Play Defend (card 2)
Type "2"
Enter
Sleep 1s

# End turn — enemy acts, new hand drawn
Type "e"
Enter
Sleep 2s

# Play a card on turn 2
Type "1"
Enter
Sleep 1s

# Win the combat via debug shortcut
Type "win"
Enter
Sleep 2s

# Card reward screen — pick card 1
Type "1"
Enter
Sleep 2s
