Basic program for binding controls quickly in Wonderful World and other games with bad controller support at offline events.

You can specify a configuration file with -f \<PATH\> to change what game it is binding controls for, or else it will load bindings.json in its own directory (and if there is none, it will load the configuration for Wonderful World).

Currently this uses a pre-release version of enigo that has support for the numpad on Linux. This will likely need to be changed back upon the next engio release.

Current configuration files in the repo:
- 2D Fighter Maker 2nd (Generic Bindings)
- 2D Fighter Maker 95 (Generic Bindings)
- Battle Fantasia
- Comic Party Wars 2
- The Queen of Heart '99
- Touhou 3: Phantasmagoria of Dim. Dream
- Wonderful World

Let me know if there are any you want added (or made your own and want them on the repo) or issues you have discovered.

More features coming soon™.

----

## Game-specific tweaks
Some games ship with default keyboard mappings that may not interact nicely with the keyboard interaction library shipped with this program or the gane itself. This requires some user input, see changes below:

- AzuFight Taisen Shiyo: Reset keyconfig to default by removing "keyconfig.dat" from the game folder
- Battle Fantasia: Set both players to use Keyboard input. Keep all mappings to default, **except** changing Player 2 "Start" to "Numpad +", from "Numpad Enter".
- The Queen of Heart '99: Keep default controls, **except** changing Player 4 "Left" to "B", from "Delete" because delete is a bind to go back to the main menu. The keybinding screen can be glitchy, I needed to press "0" to swap it to "B".
