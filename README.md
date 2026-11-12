# Melody Quack

A cosy little game about two animal friends making music in the woods. One of you stays home and plays, the other fetches sticks from the forest, and then you sit by the campfire and jam together. Written in Rust with [macroquad](https://macroquad.rs), rendered as chunky low-res 3D at 320×240 for that early-3D-console feel. Every block, flame and meow is generated in code — no models, textures or audio files besides the icon.

## Play

- **In the browser:** open the GitHub Pages link for this repo
- **Locally:** `cargo run --release`
- **With your Serato tracks on the decks:** start the bridge and open http://localhost:4747 (see [Live](#live-show-your-serato-tracks-on-the-decks))

On a phone or tablet, the web version starts with a simpler set of controls: a stick on the left for movement, six note buttons and a large E action button on the right, plus a mode switch. Tap **more** to reveal recording, loop clearing, character selection, help, building tools, echo and tape; tap **less** to hide them. Shorter portrait screens use smaller buttons. Turn the phone sideways for the best view.

**Sticks and stones.** A few random trees have a stick lying at their foot, marked with a little sparkle. Whoever gets there first (player 1 with E, player 2 with O, or your friend) picks it up, and a new stick appears somewhere else after a while. Carry it to the logs by the fire. E or O on a rock picks up a stone. Trees without a stick get shaken for acorns instead.

## Characters

Press V (or start the game) to choose who you are, and who your friend is. Each one has its own voice, so the six note keys sound completely different:

| Character | Voice |
| --- | --- |
| cream cat | the classic soft meow |
| russian blue | a gentle, muffled meow (with yellow-green eyes) |
| ginger tabby | a bright, short meow |
| black cat | a deep meow, an octave lower |
| labrador | a round woof |
| cream pup | a little yap (no tail, teal collar) |
| shiba | an "awoo" howl |
| duck | a buzzy quack |
| owl | a vocoder "hoo-hoo", like a vocal machine, with headphones on |
| fox | a hoarse, breathy bellow (with a teal scarf and a big bushy tail) |
| sea otter | smashes a shell on his belly with a rock, so every note is a tuned clack |

In the picker, W/S picks the row (you or your friend), A/D changes the character, and E starts.

On a fresh world, a short first-play guide helps you sing your first note, visit the campfire with your friend, and earn a star from playing together. The guide is not shown when loading an existing save and starts again if you create a fresh world. Your saved game format is unchanged.

## Controls

| Key | Action |
| --- | --- |
| P | switch between **1 player** and **2 players** (split screen) |
| V | choose characters |
| WASD | walk player 1 (arrows work too in 1-player mode) |
| 1 – 6 | sing on D, E, F#, A, B, C in your character's voice (and cast a spell in spell mode) |
| E | player 1: interact, sit, cook, cast and catch fish, build, use home furniture or open the catalogue |
| X | player 1: switch building tools outdoors; inside, rotate or move furniture or close the catalogue |
| IJKL or arrows | walk player 2 (2-player mode) |
| 7 8 9 0 - = | player 2 sings on the same six notes |
| O | player 2: interact, sit, cook, cast and catch fish, build, use home furniture or open the catalogue |
| U | player 2: switch building tools outdoors; inside, rotate or move furniture or close the catalogue |
| Tab | switch between **spell mode** and **music mode** |
| R | start / stop recording into the loop |
| C | clear the loop (in 2-player mode only player 1's notes) |
| Y | player 2: clear only player 2's notes from the loop |
| Shift+C | clear the partner's songbook: he forgets every tune and starts writing from scratch |
| Q | quantise: **off** / **soft** / **tight** |
| B | beat on / off (kick, clap and hats) |
| F | follow: listen through the microphone and lock the tempo to the music around you. A blinking amber light means syncing, a green light means synced (the browser asks for the microphone the first time) |
| G | echo: every note repeats on the beat (dotted eighths), each repeat softer and darker, like a dub delay |
| T | tape: start recording everything you hear (you, your friend, beat, echoes); T again saves it as a .wav |
| N | press twice to start a fresh world |
| click the game / Esc | pause, with a mixer: you, your friend, beat & pot, world and master volume (tap the game on a phone). The pause menu also has **restart the game**: press it (or R) twice and the game saves, then starts again from your save |
| H | show / hide help: three pages (basics, music, world), left/right arrows or tap a tab to switch, H or Esc to close |
| M (or Enter) | open / close the bag: your things, today's quests and stored furniture (player 2: Backspace) |

### Home and the campfire

The cat starts at home by his log cottage. While he makes music, the dog trots off into the forest, sniffs around the trees and comes back with a stick in his mouth. The first stick lights the campfire, and he keeps fetching until there's a good pile.

Walk up to the fire and press E to sit on the log. The sky slowly turns to dusk, the cottage windows and lanterns glow, and the fire lights up everyone nearby. Once the dog has brought enough sticks he comes and sits on the other log and plays along: a soft woof bass line that follows the chords, a shaker, and little echoes of whatever the cat just meowed, an octave lower. Move to stand up.

Your friend doesn't just follow along. Like a little AI bandmate, at the start of every two bars he decides whether to take a turn: sometimes he plays bass under you, sometimes he improvises his own phrase, wandering around the scale and now and then borrowing notes you just played. He's more likely to answer when you've just played something, and he won't hog the spotlight for long. When he solos, it's your cue to answer back.

He also writes songs as you go. A fresh game starts with an empty songbook, so his first tune is only a couple of notes, built from scratch out of what you just played. Each time he solos he either writes a new tune, grows one he already knows (adds a note, bends one, shifts it up or down), or plays a favourite again. Tunes get longer the more you jam together, up to eight in his songbook. The songbook is saved with the game. Shift+C makes him forget every tune and start again from scratch, and so does a fresh world (N twice).

Each stick burns for about 45 seconds, so the dog heads back to the forest when the fire gets low. Ember (1) on the fire relights it, and rain (3) nearby puts it out.

### Building together

The dog doesn't only fetch firewood. Once the fire has enough, he brings wood from the trees and stones from the rocks, and drops them by the fire. Walk around the yard, face an empty square and press E to build: wood walls, stone walls (stack up to three high) and plank floors. X switches tools, and remove gives the material back. Every block plays a note as it goes up, a marimba for wood and a bell for stone.

### Fishing on the beat

Walk to the end of the dock and press E (player 1) or O (player 2) to cast. A ring shrinks around the bobber and closes on every second beat. Press E as it touches the bobber (you have about a fifth of a second either side), three times, to land one of 120 kinds of fish, from a bluegill to a dragon koi.

- **Three tries:** a miss costs one try; the fish only gets away after three misses. Pressing far too early locks you out of that ring, so mashing E does not work.
- **Feedback:** a little "perfect!" (with sparkles) or "nice!" pops up right above the ring; "too early" appears when you jump the gun.
- **A bit unpredictable:** after the lesson, each ring gets a slightly tighter window by a random amount, and some rings start smaller or wait a little longer before closing, so you have to watch rather than count.
- **Your first fish** comes with a little lesson from your friend: a card shows how the ring works, a big E! or O! prompt pops up for the player who cast, and you cannot lose that fish, however many you miss.
- **Co-op (2 players):** either player can start fishing from the end of the dock. The other can join from nearby, and both play the same rings. Either player hitting a ring counts, so you cover for each other. Hit the same ring together ("together!") and it counts double.
- **With your friend (1 player):** when your friend sits on the dock with you, they sometimes tug the line and save a ring you missed, and sometimes catch a fish of their own.

The beat comes from the loop tempo, so with F (mic sync) you can fish in time with the music you're listening to.

### The fishdex: 120 fish

Every fish you land goes into the **fishdex**. There are 120 species, each with its own number (#001 to #120), pixel picture and rarity:

- **common** (#001-#050), **uncommon** (#051-#090), **rare** (#091-#112) and **legendary** (#113-#120, the koi family and the big ocean fish)
- perfect timing makes rarer fish more likely; hitting rings together in 2-player mode helps too
- a card pops up for every catch, with a "new!" tag the first time

Open it from the pause menu (Esc, click or tap the game, then **fishdex**, or F while paused) or from the catalogue in your house (the third tab). Left/right arrows (or Tab) switch between fishdex and mushrooms, and from the catalogue on to the diary and catalogue tabs; on a phone, tap the tab names. Type a name or a number to search ("koi", "#102", "17"); on a phone, tap the search bar. Fish you have not caught yet show as a dark shape with "???", and name searches only find fish you have caught.

### Mushrooms: 120 to find

Mushrooms grow on the grass around the trees. Walk up to one and press E (player 2: O) to pick it.

- **Seven a day:** each real day, seven of the 120 species grow, picked at random for that date. Come back tomorrow for a different seven. Rain makes them grow faster, and up to ten grow at once.
- **The mushroom dex:** every species has a number (M001 to M120), a pixel picture and a rarity (common, uncommon, rare, legendary, from the button mushroom to the melody truffle). It is the fourth tab next to the fishdex.
- **Look, don't eat:** some, like the fly agaric, are logged in the dex but never go into the basket. In real life, never eat a wild mushroom an expert has not identified.
- Edible ones go into your basket for cooking (see the bag with Enter).

### Real time, seasons and weather

The world follows your real clock: morning, day, a pink-orange sunset, dusk and night with stars and a moon, in the world and inside your house and loft (the rooms get darker at night unless the lamps are on).

- **Seasons** follow the date: blossom-pink trees in spring, green in summer, orange in autumn, darker in winter.
- **Weather:** clear, cloudy, rain (falling streaks, more mushrooms) or snow (flakes and white ground). On localhost:4747 and GitHub Pages the game asks for your location (or uses Hamburg if you say no) and reads the live forecast from [Open-Meteo](https://open-meteo.com), free and without an account, every 20 minutes, including the exact sunrise and sunset. The game shared as a Claude link can't reach websites, so there the weather is made up to suit the season, and the desktop version does the same.
- The time, season, weather and temperature show under the top bar.

### Pixel emoji

Send a little pixel emoji bubble above your head: **Z** for player 1, **.** (full stop) for player 2, or the **emoji** button on a phone. Press it again to step through the 12 emojis (heart, smile, laugh, wow, sad, sleepy, music, star, fish, acorn, fire, huh); after a second it sends. You can also click or tap one in the row. Playing alone, your friend answers with an emoji of its own.

### Cooking percussion

While you're sitting by the fire, press E or O again and the pot goes on. Either seated player can cook on the beat. Keys 1 to 4 for player 1 and 7 to 0 for player 2 become a kitchen drum kit: 1 kick, 2 pot lid, 3 chopping, 4 sizzle. They record into the loop like any other note. Every hit that lands on the beat fills the pot, and when it's full you get a dish depending on what you caught: campfire tea, grilled fish, fish stew, or golden koi soup. Collect all four. The dog keeps playing bass while you cook, and you both get little hearts when it's ready.

### Two players

Press P and the second animal is controlled by another player. The screen splits down the middle so each of you gets your own camera and shares the same world, inventory, house, music loop and diary. Both players can gather sticks, stones and acorns, build, sit by the fire and cook on the beat. Either player can cast a line at the dock; the other can help catch fish, and hitting a ring together counts double. Either player can enter or leave the cottage, bringing both players through the shared doorway. Inside, player 1 uses E and X and player 2 uses O and U to interact with all furniture, sit on a sofa or bench, lie on a bed, play instruments, water plants, serve food, and move or place items. Both players can sit or sleep at the same time, with their positions offset so they do not overlap. Both can access the shared catalogue. Player 2 uses J/L for tabs, I/K for selection, O to choose and U to close. The house holds one piece of furniture at a time while it is being moved or placed. The two DJ decks also remain available for b2b. Player 1 notes show in colour on the loop grid; player 2 notes in golden brown. Player 2's cooking hits use newly appended note IDs (28-31), so Y clears them without touching player 1's percussion. Existing note IDs, furniture indices, and saved lines are unchanged. The two-player mode is designed for keyboard-controlled local co-op; mobile touch controls for two players have not been validated.

### Spells

| Key | Note | Spell |
| --- | --- | --- |
| 1 | D | **ember** — set the tile ahead on fire |
| 2 | E | **bloom** — ash → grass → flowers → sprout → tree |
| 3 | F# | **rain** — soak the area around you and put out fires |
| 4 | A | **gust** — wind in the direction you face; fire races downwind |
| 5 | B | **frost** — freeze water into walkable ice |
| 6 | C | **sun** — dry everything nearby and melt ice |

### Looping

The loop is two bars of 16th notes. Press R and play: every meow snaps to the nearest 16th and repeats on the next pass, and recording again layers more notes on top. Loops play in both modes, so you can build a groove in music mode and then go back to exploring while it plays. Notes coming from the loop never cast spells.

### Quantise

Q cycles how strictly your timing gets corrected in music mode:

- **off** — pure you: notes play the instant you press and loop exactly where you played them
- **soft** (default) — live notes are untouched, recorded notes get pulled halfway to the nearest 16th, so the loop tightens up but keeps your feel
- **tight** — recorded notes snap fully to the grid; live notes within 30 ms after a 16th play instantly, anything earlier waits for the next 16th so it lands on the beat

### Mic sync

Press F and the game listens through your computer's microphone. It finds the onsets in the sound (mostly the kick), works out which beat length repeats most strongly between 80 and 160 BPM, and then nudges the loop so its downbeats land on the music's beats. It usually locks within about five seconds. The loop, the beat and the cat's head-bob all follow, so you can play a record and meow along in time. Press F again to stop listening and keep the tempo it found.

While it listens, a light by the loop shows how it is going: blinking amber with "syncing..." while it searches, green with "synced 128 bpm" once it has locked on. It works in the browser too; the first press asks for microphone permission. On macOS the desktop version asks for permission for your terminal app.

## The chemistry

Nothing is scripted. Every tile follows a few rules, and puzzles come from how they combine:

- Fire spreads to dry grass, flowers and trees, faster downwind, and leaves ash
- Wet tiles don't burn, and fire on a wet tile turns to steam
- Fire next to ice melts it back into water
- Ash slowly grows back into grass
- Water blocks your path until you freeze it

## Home: decorate with acorns

Walk up to the cottage door (or press E there) to go inside. The room is yours to furnish, and your friend comes in with you and pads around.

**Earning acorns**
- shake a tree in the forest with E, or O for player 2 (each tree needs a rest before it drops more)
- jam by the fire: every loop where you play along earns one
- catch fish (2 for common, 4 uncommon, 8 rare, 15 legendary) and cook dishes (5, or 10 for a new one)
- your friend brings acorns home when there is nothing else to fetch, and fishes next to you on the dock, sometimes catching one of his own

**Decorating**
- E or O at the lectern in the back corner opens the catalogue and the diary: player 1 uses A/D tabs, W/S selection, E to buy and X to close; player 2 uses J/L tabs, I/K selection, O to buy and U to close
- E / O places what its player bought, X / U turns it. Face a piece and press X / U to pick it up and move it, or press E / O at the catalogue while holding it to sell it back for half
- wallpaper (cream, mint, rose, night) and floors (oak, cherry, checker) cost 10 acorns once, then switch for free

**Furniture that makes music**
- piano: E or O to sit; player 1 plays with 1-6 and player 2 with 7-=
- drum kit: E or O to sit; 1-6 / 7-= play kick, clap, hat, lid, chop and sizzle
- record player: plays your friend's tunes from the songbook while you're home (E to switch it off)
- wind chime: rings by itself now and then
- speaker: makes the echo (G) bigger, inside and out
- lamp: warm light

**Brutalist / club corner**
- concrete wall and concrete floor (board-formed, with tie holes and seams)
- dj booth: two turntables and a mixer. Face it and press E to get on the decks: the view goes first person and a 132 bpm techno beat starts
  - 1-6 play pads: kick, clap, hi hat, open, low tom, rim
  - W/S tempo (80-170 bpm), A/D pattern (techno, house, breaks, classic, off), R records your pads into the loop, X leaves
  - in 2 player mode, player 2 walks up and presses O to go b2b: 7 8 9 0 - = play bass, chord, vox, shaker, bell, snare, O leaves
  - the records only spin while the beat is on, and the track name from the radio or the live Serato bridge shows on the deck panel
- subwoofer stack: the kick gets heavier, and the cones pump with the beat
- red light: the whole room goes dark red and throbs on the beat
- concrete bench and steel pillar
- **techno club set** (free, at the bottom of the catalogue): one tap turns the room concrete, fills it with the club pieces that fit, and starts the beat at 132 bpm. Switch the wall and floor back any time, since styles you own are free

Everything you buy and where you put it is saved.

### Make your home bigger

The first row of the catalogue sells the next step for your home:

1. **bigger room** (80 acorns): 9 x 6 tiles
2. **even bigger room** (160 acorns): 11 x 7 tiles
3. **penthouse loft** (250 acorns): a staircase at the back right leads up to a loft along the back wall, with a glass railing and a floor-to-ceiling window over a harbour city
4. **bigger penthouse** (400 acorns): 11 x 9 tiles, and the loft upstairs is twice as deep (4 rows instead of 2)

The city outside the penthouse window follows the real clock and weather: blue sky by day, a pink-orange sunset, lit windows, stars and a moon at night, clouds, rain or snow falling past the glass, and red lights blinking on the harbour cranes after dark.

Your furniture stays where it is when the walls slide out. With the penthouse, anything along the back wall ends up on the loft. Walk up the stairs to reach it, and place furniture up there like anywhere else. The tile at the top of the stairs stays free so you can always get off. If something stands where the stairs go, or across the loft edge, the catalogue asks you to move it first.

### Using your furniture

Face a piece and press E (player 1) or O (player 2):

- **lamp** and **red light**: switch the light on or off. Every light that is on glows and lights up the room around it, so several lamps make the room brighter
- **bed**: lie down under the quilt and sleep; the screen fades out and the game saves. Player 1 always takes the left side and player 2 the right. Playing alone, your friend hops in on the right with you
- **sofa** and **concrete bench**: sit down; move to get up
- **plant**: water it once a day; it grows twice and then blooms. Miss a whole day and it withers (brown, drooping); water it again to bring it back
- **bookshelf**: pick one of six books (experience design, software development, electronic music, AI, UI/UX design, and Melody Quack's own cook & quack) with A/D and open it with E. You read it in first person, an open book in your paws: A/D turn the pages, X goes back to the shelf, X again leaves. Each book has 10 pages of short true stories. On a phone, tap the left or right side to choose or turn, the middle to open
- **table**: serve one of the dishes you have cooked by the fire; press again for the next one, and once more to clear the table
- **speaker** and **subwoofer**: switch them on or off (bigger echo, heavier kick)
- **steel pillar**: knock on it for a metal clang
- **rug**: a little dance
- **piano**, **drum kit**, **record player**, **wind chime**, **dj booth**: play them, as before
- **mirror ball** (300 acorns): hangs from the ceiling, spins and throws coloured light spots across the room; E switches it on or off
- **dance arcade** (500 acorns): stand in front and press E to dance. Arrows in four colours scroll up the right side of the screen; step when one reaches the outline at the top. Player 1 uses the arrow keys or WASD, player 2 joins with O and steps with J K I L. X (player 2: U) or Esc stops. The arcade turns on the microphone: kicks become left arrows, snares and claps down, melodies right and hi-hats up, placed one bar ahead on the beat grid so your steps land in time with the music. With no mic (or silence), the arrows follow the game's own beat and your loop. Timing gives perfect, great or good, and a combo builds up
- **wall tv** (1000 acorns): hangs on the wall behind its spot. E switches through five pixel visual channels (spectrum, tunnel, plasma, rain, scope) and off. They move with the music from the microphone when it is on, otherwise with the beat and your notes
- **sampler** (180 acorns): E opens a step grid with four rows (kick, clap, hat, your animal's voice) and 16 steps that play in time with the beat while you're in the house. Arrow keys or WASD move, E turns a step on or off, X or Esc closes (player 2: IJKL, O, U). Space records 1.5 seconds from the microphone into the selected row so it plays your own sound; Space again goes back to the built-in sound. Record only your own voice or sounds: recordings stay in memory on this device while the game is open, are never uploaded, and are left out of tape (.wav) exports. Your step pattern is saved

### Putting furniture away

- While carrying a piece (X on it), press **M** to put it in your bag.
- Open the bag (M) inside the house: W/S choose a stored piece, E takes it out to place it, and X twice packs **all** furniture into the bag.
- When you buy a bigger room, any furniture in the way of the new stairs or loft edge goes into your bag automatically instead of blocking the upgrade.

### Daily quests

Every real day brings three random quests, like catching fish, picking mushrooms, cooking, watering a plant, picking up sticks, singing notes, sending emojis, earning a star or dancing on the arcade. They're listed in your bag with their progress and acorn reward, and pay out the moment you finish them. Quests that need a plant or the dance arcade only appear once you own one.

## Stars and the diary

Jam together by the campfire (both players in 2P, or you with your partner in 1P) and keep the notes coming. A timer shows how long you've played together:

- 30 seconds: 1 star
- 1 minute: 2 stars
- 3 minutes: 3 stars

Short pauses are fine. When you stop for more than 8 seconds, the jam is written in the diary: the date, who played, how long, and the stars. The diary is the second tab on the lectern inside the cottage (A/D switches tabs). Every jam is recorded note by note, so pick one in the diary, press E, and the record player plays it back. The last 10 jams keep their recordings in the save.

## Radio: play your mixes in the game

On the web version a radio sits under the game. Pick a station and press play on the player. The track name shows under the game and in the game's top bar, with a "now playing" message whenever the track changes, so everyone listening knows what's on.

- **SoundCloud:** a profile, a playlist or a single track. The player says which track is playing, so names come through by themselves.
- **Mixcloud:** a mix plus a tracklist with start times. The game follows the mix's position and switches the name at each start time.
- **BPM:** give a mix a BPM and the game's beat, echo and fishing lock to it.

The station list everyone sees lives in `web/sets.js`:

```js
window.MELODY_SETS = [
  { title: "cayennepfeffa on SoundCloud", source: "soundcloud", url: "https://soundcloud.com/cayennepfeffa" },
  {
    title: "my mixcloud mix",
    source: "mixcloud",
    url: "https://www.mixcloud.com/USER/MIX-NAME/",
    bpm: 128,
    tracklist: `0:00 artist - title
4:12 artist - title
1:02:30 artist - title`
  }
];
```

Listeners can also add a mix for themselves under "add a mix" (kept in their browser). The radio needs the players from soundcloud.com and mixcloud.com, so it works on GitHub Pages or any normal web host, not inside pages that block other sites.

## Live: show your Serato tracks on the decks

While you DJ in Serato, a small script on your laptop (the **bridge**) reads which tracks are loaded on your decks and shows them in the game: in the top bar, under the game, and on the panel when you stand at the DJ booth in your house (deck 1 on the left, deck 2 on the right).

The track names only reach two places:

| where you play | shows your tracks? |
| --- | --- |
| **http://localhost:4747** (the game served by the bridge on your laptop) | yes |
| your GitHub Pages site | yes, and anyone else watching it sees them too |
| the game shared as a Claude link | no: that page is not allowed to talk to your laptop or to other sites |
| the desktop version (`cargo run`) | no |

### Step by step (Mac)

**Once, before your first time**

1. Check that Python 3 is there: open Terminal and type `python3 --version`. If a version number comes back, you are set. If macOS offers to install the developer tools, say yes and wait for it to finish.
2. Unzip the newest `melody-quack.zip`. If you unzip a newer copy later, use the new folder: the bridge serves the game from whichever folder you start it in.

**Every time you play**

1. Open Serato and load a track on a deck.
2. Open Terminal and go into the game folder (or type `cd `, drag the folder into the Terminal window, and press enter):
   ```
   cd ~/Downloads/melody-quack
   ```
3. Start the bridge and leave this window open while you play:
   ```
   python3 bridge/serato_live.py
   ```
   It prints `now playing: ...` with one line per deck whenever you load a track.
4. Open **http://localhost:4747** in Safari or Chrome. If you have played it before, reload with cmd+shift+R so you get the newest version.
5. In the game, go into your house, walk up to the DJ booth and press **E**. The panel above the turntables shows what is loaded on each deck in Serato.
6. When you are done, click the Terminal window and press **ctrl+c**.

**If no track names show up**

- Make sure the address bar says `localhost:4747`, not a Claude link.
- Check what the bridge can see in Serato:
  ```
  python3 bridge/serato_live.py --debug
  ```
  It lists the decks it found. If it finds nothing, send that output to whoever helps you.
- `can't open file .../bridge/serato_live.py` means Terminal is not in the game folder: do step 2 again.
- If your Serato library is somewhere unusual, point the bridge at it with `--database /path/to/master.sqlite` (Serato DJ Pro 4) or `--serato /path/to/_Serato_` (older versions).

### Recognise what plays on your speakers, and get track ideas

The bridge can also listen to your laptop's microphone, recognise the song coming out of your speakers (the same way the Shazam app does), and show:

- the **artist** and **track**, with links to find them on SoundCloud and Shazam (under the game, below the decks)
- **goes well with**: a short list of tracks that fit with it, on the DJ booth panel and as SoundCloud links under the game

It works with or without Serato. Without Serato running, the heard song becomes "now playing".

**Once, before your first time**

1. This needs a newer Python than the one that comes with the Mac. With Homebrew: `brew install python@3.12`, then install the helpers with `python3.12 -m pip install --break-system-packages shazamio sounddevice` and skip to step 3. Check with `python3 --version`: it must say 3.10 or higher. If it says 3.9, install Python 3.12 from [python.org/downloads](https://www.python.org/downloads/) (open the .pkg and click through, then double-click **Install Certificates.command** in the window that opens), then close and reopen Terminal and check with `python3.12 --version`.
2. Install the two helpers:
   ```
   python3.12 -m pip install shazamio sounddevice
   ```
3. The first time the bridge listens, macOS asks whether Terminal may use the microphone. Click **Allow**. If you clicked no by mistake: System Settings → Privacy & Security → Microphone → turn on Terminal.

**Every time**

Start the bridge with the newer Python: `python3.12 bridge/serato_live.py` (plain `python3` may still be the old 3.9, which shows Serato but can't listen). It prints `listening to your speakers every 20 seconds` and then, a few seconds into a song, `heard: ...` and the `goes well with: ...` lines.

**Two switches: DJ set mode and karaoke**

Both live in the pause menu (Esc: **J** for DJ set, **K** for karaoke) and as buttons above the decks under the game. The game remembers them.

- **DJ set mode off** (the default): the game shows the song heard on your speakers as a small yellow tag over the game, in the top bar and on the booth panel. Serato's decks stay hidden.
- **DJ set mode on**: the top bar and the booth panel show your Serato decks in two columns, as before, with the heard song in blue underneath.
- **Karaoke on**: when the heard song has synced lyrics, they appear over the game like subtitles, filling in yellow as the line is sung, with the next line underneath. At the DJ booth they move to the top of the screen; they hide in the pause menu and the fishdex.

How karaoke keeps time: Shazam reports how far into the song your recording was, the lyrics come from [LRCLIB](https://lrclib.net) (free, community-made, with timestamps), and the page counts on from there. Every recognition (every 20 seconds) re-syncs it, so if you pitch a track up or down it drifts a little and then snaps back. Not every song has synced lyrics; Terminal says `karaoke lyrics: none found for this song` when there are none. Lyrics only show on localhost:4747, not on GitHub Pages.

**What Terminal tells you**

- `*** listening to your speakers is OFF ***`: the bridge was started with the old Python, or the helpers are missing. It says which, and the exact command to type.
- `the microphone hears almost nothing`: turn the speakers up, allow Terminal under Microphone in Privacy & Security, or pick another input with `--device`.
- `heard sound but Shazam did not know the song`: Shazam doesn't have that track (common with unreleased promos and edits).
- `heard: Artist - Title`: it worked.

**Options**

- `--list-devices` lists the inputs it can listen on. If you DJ through headphones or an audio interface, the microphone cannot hear the music: pick the interface's input, or a loopback device such as [BlackHole](https://existential.audio/blackhole/), with `--device 2` (the number from the list) or `--device "BlackHole 2ch"`.
- `--listen-every 30` listens less often (the default is every 20 seconds, 8 seconds at a time; `--listen-seconds` changes that).
- `--no-listen` turns listening off and goes back to Serato only.

**Where the suggestions come from:** Shazam recognises the song and is asked first for similar tracks. When Shazam has none, the bridge asks Deezer's free public API for artists related to the one it heard and takes each one's top track. Terminal says which one it used (`goes well with (from Deezer)`). SoundCloud's API needs a registered developer app with a secret key and has no "related tracks" feature, so every artist and track links to a SoundCloud search instead, one click away.

### Good to know

- Serato writes a track into its history as soon as you load it, so a deck shows the new name when you load it, a little before you mix it in.
- The bridge reads both kinds of Serato history: the library database of Serato DJ Pro 4 (`~/Library/Application Support/Serato/Library/master.sqlite` on a Mac, `%LOCALAPPDATA%\Serato\Library` on Windows) and the older `Music/_Serato_/History/Sessions` files.
- The bridge also sends each track to anyone with your GitHub Pages site open, through [ntfy.sh](https://ntfy.sh) (free, no account). Add `--offline` to keep it on your laptop only, for example while you practise. The channel name is in `web/sets.js` (`MELODY_LIVE.topic`); it works like a private radio frequency, so keep it to yourself or change it in both places.
- Put a stream link (Mixcloud Live, Twitch, YouTube) in `MELODY_LIVE.listen` in `web/sets.js`, and a "listen live" link appears next to the track on GitHub Pages.
- The game only shows the names. The music itself plays from Serato.

## Saving

The game saves by itself every few seconds and when you close it, and picks up where you left off: your walls and floors, wood, stone, fish, dishes, both characters, the loop, the beat, the tempo and the echo setting. Press N twice for a fresh world.

- desktop: the save lives in `~/.melody-quack/save.txt`
- browser: the save lives in that browser's local storage

Press T to roll the tape and T again to stop. Your jam is mixed into a 44.1 kHz .wav:

- desktop: `~/Music/melody-quack/jam-01.wav`, `jam-02.wav`... (or `~/melody-quack-jams/` if there is no Music folder)
- browser: downloaded as `melody-quack-jam-01.wav`

A tape stops and saves itself after 5 minutes, or when you close the game.

## How it's made

- The world renders into a roughly 360×270 texture with depth, then scales up with nearest-neighbour filtering for crisp pixels, UI included
- Everything is built from coloured boxes every frame: the cat, the dog, the cottage, the campfire, the dock, your walls, even the fishing line and the ring around the bobber
- Blocks are shaded per face from a fixed light direction, with distance fog for a soft, misty look
- Lighting is done per box: an evening tint at dusk plus a flickering warm light from the campfire
- The 3D world renders at roughly 360×270 for the chunky look, but all text and menus are drawn afterwards at full window resolution with a soft shadow, so they stay sharp and easy to read
- Trees and tall walls between the camera and the cat turn see-through so you never lose sight of her
- Each meow, woof, drum, pot lid, sizzle, splash and fire crackle is synthesised at startup and played through a tiny built-in mixer on desktop, or the browser's Web Audio on the web
- Mic sync finds onsets in the low end, autocorrelates them to find the tempo, then phase-locks the loop to the detected beats
- Pushing to `main` builds the WebAssembly version and deploys it to GitHub Pages


## recent changes

- Furniture can go into the bag (one piece or everything), and room upgrades store pieces in the way; M opens the bag. Daily quests for acorns. Plants need water once a day.
- 3D preview in the character picker, a sampler with a step grid and your own mic sounds, the techno club set (renamed), and emoji pixel fixes.

- Bigger catalogue preview; a paged, easier-to-read help (H or Esc to close); E picks up a stick or mushroom before shaking the tree.
- New furniture: mirror ball, dance arcade (DDR on four mic-driven lanes) and wall tv with five visual channels.
- Bigger penthouse upgrade with a deeper loft, and the penthouse window city follows the real clock and weather.

- The emoji picker draws crisp, even pixels on a light card, and the question mark emoji is black.

- 120 mushrooms (seven species a day, near trees), mushroom dishes, a mushroom dex tab, and tab switching that works from the dexes.
- Real-time day and night with sunsets, seasons, and live local weather (rain, snow, clouds, clear).
- Trees only block at their trunks, so you can walk between and under them.
- Fishing: small perfect/nice pops above the ring and slightly less predictable rings. Three stars now take 3 minutes of playing together.
- The Claude link shows the game only, full screen. The cook & quack book joins the shelf, and books are read from straight above.

- The bookshelf opens a first-person reading view with five books of facts on 3D page spreads.
- The catalogue's 3D preview is no longer upside down.

- DJ set mode (J) and karaoke (K) switches; the heard song shows as a tag over the game; synced karaoke lyrics from LRCLIB fill in as they are sung.
- The DJ booth panel shows the heard song in blue, and suggestions fall back to Deezer's related artists when Shazam has none. The bridge explains in plain words when listening is off, the mic hears nothing, or it is already running.
- The bridge can recognise the song playing on your speakers, shows the artist and track with SoundCloud links, and suggests tracks that go well with it, on the DJ booth panel and under the game.

- In the house, 2-player mode now stays on one shared camera instead of splitting the screen.
- Enter opens the inventory bag on keyboard, and the web touch controls have a bag button in the expanded panel.
- Cooking and gathering now have first-time onboarding cards, and their lesson flags are saved after the original fishing lesson flag.

- The bed is now a wider double bed, with a softer sleeping pose for lounging together.
- The table is now a longer family-style dining table.
- The catalogue now shows a floating 3D furniture preview with description text underneath, and uses left/right browse controls while up/down switches tabs.

- The catalogue 3D preview now renders into its own texture before the screen UI is drawn, so opening the shop no longer redirects the UI to an off-screen render target.

## Catalogue smoke test

1. Compile and run the updated Rust code. The prebuilt `web/melody-quack.wasm` is from an earlier build and does not contain these changes.
2. Enter the house, stand near the catalogue in the corner and press E. The catalogue heading, 3D furniture preview, price and item description should all appear.
3. Press A/D to browse items, W/S to switch between catalogue and diary, and X to close. Check that the scene, buttons and labels are still visible after closing.
4. Press P to enable two-player mode. Use Player 2's I/J/K/L keys to reach the catalogue, O to open it, J/L to browse, I/K to change tabs, and U to close.
5. Check that the double bed, family table, inventory and save/load behaviour still work. Repeat at a narrow browser window size after rebuilding WebAssembly.

Run the source-order regression checks with `python3 -m unittest discover -s tests -v`. These checks do not replace a real Rust build or an on-screen gameplay test.

## The PC bang and the chapel rave

- **PC bang:** the dark building with the neon sign, east of the cottage. Walk to the door and press E. Sit at any computer (E) to use Quack 95.
  - **rave.exe** books a rave: type a name, pick a sound and tempo, and you get a code and a flyer.
  - **quacknet** is the town's web. A tiny dot at the bottom of the home page leads underground, where the flyers hang.
  - **terminal** hides a small capture the flag. Type `help` to start; the flag is the code of a secret rave.
  - Mr. Kim at the counter has opinions.
- **St. Bassilica:** the concrete chapel at the end of the forest path. The board outside carries flyers (E to read, A/D to flip); old torn ones have codes that no longer work.
- **The goose** at the door wants a code. Type it, or click one you have already seen.
- **Inside:** a generated techno set seeded by your rave, a duck on the decks and a dancing crowd. WASD to move, 1-6 to sing over the set, Esc or the door to leave. When the radio is playing a live set, the chapel shows that instead of its own.

## Raving together online

Friends can meet in the same chapel rave over the internet. It needs one small server that everyone points at.

- **Run the server:** `cd server && cargo run --release` (port 7777, or `--port N` / the `PORT` variable). `server/Dockerfile` builds it for any host that runs containers. It only relays messages between players in the same rave; it keeps nothing on disk.
- **Point the game at it:**
  - Browser: set `server` in `web/sets.js`, for example `server: "wss://quack.example.com"`. A page served over https needs a `wss://` address.
  - Desktop: put the address in `~/.melody-quack/server.txt`, or set `MELODY_QUACK_SERVER`.
- **Host:** book a rave with rave.exe as usual. While you are connected it is announced to the server, so its code works for everyone.
- **Join:** give the goose a friend's code. He checks the list.
- **Inside:** friends carry a small turquoise marker above their heads, so you can tell them from the crowd. You see each other move and hear each other sing, and the set is synced so everyone hears the same bar. The corner of the screen shows how many are here.
- With no server set, or when it cannot be reached, everything works offline as before.

## Your name, the post and the secret rave

- **Your name:** the game asks for it once, before anything else. Friends see it above your head in the chapel; a second player on the same keyboard shows as "name +1". Both players are sent to friends.
- **The envelope:** click it for the goose's announcement about tonight's secret rave, with a countdown to the millisecond. A red 1 means there is news.
- **The first 30:** the goose writes the code down exactly 30 minutes before doors. Sit down at a PC bang computer as close to that moment as you can; a banner in the PC bang counts down to it. You get one attempt, it has to be within ten seconds either side, and the game tells you how far off you were. The 30 closest are on his list.
- **The other 70:** five minutes later the terminal gives the code to anyone, until 70 more have it.
- **Getting the code:** in the terminal, `su goose` (the password is on the underground page), then `cat .rave`, then `rot13` what it gives you.
- **Doors:** the goose lets nobody in before the start time, and the chapel holds 100 players at most, both of you counted if you are two.
- **Offline:** with no server there is no schedule; the terminal gives you a secret rave of your own, as before.

The server picks the start time: by default every four hours on the hour (UTC). `--rave <unix seconds>` sets one exact time, `--every <minutes>` changes the rhythm, and `--rave-in <seconds>` is for trying things out.

## Becoming a DJ

The arena has a membership for DJs: 5 euros a month on Patreon. Patreon takes care of the payment, the receipts and cancelling. Every paying member gets a DJ key from the admin.

- **Where you type the key.** On the very first screen, and again every time you press restart in the pause menu, there is a "become a dj" button under the name box. It opens a box for the key and a link to the Patreon page. The key is saved with your game.
- **Asking to play.** In the chapel, walk up to the booth and press E for the decks. A DJ can ask to play the rave that is running. The request goes to the admin's post.
- **The admin says yes or no in the game.** The admin types the admin key into the same box. Their envelope then has a "dj requests" page with "yes +1" and "no" next to each request. Every yes is good for one set and adds 1 euro of credit to that DJ, shown at the decks. "paid" clears a DJ's credit once it has been paid out or taken off the next month.
- **Playing.** After a yes the decks offer two ways to play. *In-game decks*: switch the beat and the melody on and off, change the sound and the tempo, and hit the six pads; everyone in the chapel hears the same thing. *Real audio*: paste the https link of your own live stream (Serato or any other input sent through a streaming service); the browser version plays it in the chapel, the desktop version shows a "listen in your browser" button.
- DJs have a purple name with "dj" in front of it.
- **The look of the room.** While live, the DJ can change the arena for everyone with the "look" button at the decks: the gothic chapel, a jungle at sunset, a terrace with a view of San Francisco (the bridge, the skyline, fog coming in), or a kinky party in black, red, neon and chains. The room goes back to the chapel when the set ends.

On the server: `--djs <file>` is the list of members, one `name = key` per line, and can be edited while the server runs. `--admin <key>` is the admin's key, `--patreon <url>` is the page the game links to, and `--credits <file>` is where the credit is kept. The same four can be set as `MQ_DJS`, `MQ_ADMIN`, `MQ_PATREON` and `MQ_CREDITS`.

The game doesn't move any money. The credit is a number the admin settles by hand.

## Chat

Everyone in the chapel can talk. Press Enter to open the chat box, type, Enter to send, Esc to close it without sending. On a phone or tablet there is a "chat" button. Messages fade after about fifteen seconds; opening the box shows the last few again.

- Names are coloured like the name tags: blue for ravers, purple with "dj" for DJs, gold with a star for the admin.
- One message every 0.6 seconds at most, 120 characters each.
- Nobody else can take the admin's name: anyone who tries shows up as "not <name>".
- The admin can type `/mute <name>` and `/unmute <name>`. A muted player still sees their own messages, so they don't notice.

On the server, `--admin-name <name>` (or `MQ_ADMIN_NAME`) is the name the admin appears under. The admin signs in with the admin key, in the DJ key box, as before.

## Downloads: macOS, Linux and Arch

The Packages workflow (`.github/workflows/packages.yml`) builds all of these on every push; find them under the run's Artifacts. Pushing a `v*` tag attaches them to a GitHub Release.

- **macOS:** `Melody-Quack-<version>.dmg`, for Apple Silicon and Intel. Open it and drag the game to Applications. It is not notarised by Apple, so the first time macOS refuses to open it: go to System Settings, Privacy & Security, and press Open Anyway, or run `xattr -dr com.apple.quarantine "/Applications/Melody Quack.app"`.
- **Linux, any distro:** `Melody-Quack-<version>-x86_64.AppImage`. Make it executable (`chmod +x`) and run it. Or unpack `melody-quack-<version>-linux-x86_64.tar.gz` and run `./install.sh`, which puts the game, its icon and a menu entry under `~/.local`.
- **Arch Linux:** `melody-quack-<version>-1-x86_64.pkg.tar.zst`. Install with `sudo pacman -U <file>`. To build it yourself, `packaging/arch/PKGBUILD` expects a source tarball named `melody-quack-<version>.tar.gz` beside it.

Linux needs ALSA, X11 and OpenGL, which desktop systems already have.

