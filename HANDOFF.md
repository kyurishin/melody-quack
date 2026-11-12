# Melody Quack: handoff for an AI assistant

Read this before changing the code. It explains what exists, where it lives, and the rules the owner (Ky) wants kept.

## What the game is

A cosy voxel music game in Rust with macroquad 0.4. You play an animal (cat, dog, duck and others) whose voice is the instrument: keys 1-6 sing six notes (D E F# A B C). Around that sit a 32-step looper, a beat machine, a campfire jam with an AI friend who learns your tunes, fishing and cooking on the beat, building, a decoratable house with a DJ booth, a radio for the owner's DJ mixes, and a live bridge that shows the tracks loaded in Serato DJ. It runs natively (desktop) and in the browser (wasm), including phones with on-screen controls.

## Rules from the owner

- Code and comments are in English only, never Korean, even when the conversation is in Korean.
- Write no comments in the code.
- The owner writes in lowercase on purpose.
- Keep the existing save format backwards compatible: append fields and lines, never reorder (for example, new furniture goes at the end of `home::ITEMS`, because saves store the index).

## Files

| file | what it holds |
| --- | --- |
| `src/main.rs` (~6.6k lines) | almost everything: audio, rendering, game loop, UI, saving, platform modules |
| `src/books.rs` | the five bookshelf books: title, cover colour, 10 pages of (heading, text) each |
| `src/mushroom.rs` | 120 mushroom species: rarity, edible, acorns, kind, code (M001), daily pick `today(day)`, `pick_from`, `look`, 12x12 `art` |
| `src/home.rs` | the house: furniture items, prices, sizes, walls/floors, placement rules, the "techno club" preset |
| `src/tempo.rs` | onset detector and tempo estimator for mic sync |
| `web/index.html` | the web page: canvas, CSS, touch controls, the JS plugin that the wasm imports, radio panel, live track listener, mic capture |
| `web/sets.js` | radio stations (`MELODY_SETS`) and the live channel (`MELODY_LIVE.topic`) |
| `web/melody-quack.wasm` | the latest built game, so the bridge can serve it locally |
| `bridge/serato_live.py` | Python stdlib script: reads Serato history, serves the game on localhost:4747 with a `/now` feed, and posts to ntfy.sh |
| `bridge/listen.py` | optional speaker recognition: records the input device with `sounddevice`, recognises it with `shazamio`, fetches related tracks |
| `.github/workflows/pages.yml` | builds the wasm and deploys GitHub Pages |
| `README.md` | the player-facing manual, kept up to date with every feature |

## Building

- Native: `cargo build --release` (macroquad window, cpal audio and microphone).
- Web: `cargo build --release --target wasm32-unknown-unknown`, then copy `target/wasm32-unknown-unknown/release/melody-quack.wasm` into `web/`. In a sandbox where the wasm target cannot be downloaded, build std from source: `RUSTC_BOOTSTRAP=1 cargo build --release --target wasm32-unknown-unknown -Zbuild-std=std,panic_abort`.
- Every `extern "C"` block for the web needs `#[link(wasm_import_module = "env")]`, and each function must also exist as `importObject.env.<name>` in the plugin in `web/index.html`.

## Architecture in main.rs

- **Platform modules** come in pairs, `#[cfg(target_arch = "wasm32")]` and native: `audio`, `mic`, `store` (save file / localStorage, jam export, restart), `radio` (now-playing text, Serato decks, bpm, touch mode, UI report to the page).
- **Web imports** used by the wasm: `store_put/len/get/remove`, `download_bytes`, `app_restart`, `radio_text_len/get`, `radio_decks_len/get`, `radio_next_len/get`, `radio_bpm`, `touch_mode`, `ui_report`, `mic_start/state/pull`, plus the `audio_*` functions.
- **Rendering:** a low-resolution render target (short side 240 px, `low_w()`/`low_h()`), a `Camera3D`, and a `Batch` of coloured boxes; everything is boxes. UI is drawn at native resolution with `ui_rect`, `ui_text`, `ui_measure`, `ui_wrap`, `ui_ticker`, `ui_star`. Split screen in 2-player mode (`half` is the half width).
- **Audio:** custom mixer; `Sound` has an id; `audio.play_ch(channel, sound, vol)` with channels 0 you, 1 friend, 2 beat & pot, 3 world, and master as mix index 4. Echo (G) is a beat-synced dub delay. The tape (T) mixes everything into a .wav. Synths are generated in code (kick, clap, hats, toms, rim, bass, piano, bells, marimba, voices).
- **Looper:** 32 steps, quantise off/soft/tight, `looper.press(id)`. Note ids: 0-5 player 1 voice, 6-11 player 2 voice, 12-15 cooking percussion, 16-21 player 1 DJ pads, 22-27 player 2 DJ pads. `note_owner(id)` and `note_slot(id)` map ids to player and key, used by the grid colours and per-player clear.
- **Beat:** `BEAT_STYLES` (techno, house, breaks, classic, off), 16-step patterns.

## Features and where to find them

- **Characters and voices:** picker on start (V), 2-player toggle (P).
- **Campfire jam and the partner:** sit by the fire (E), the AI friend learns phrases into a songbook (saved; Shift+C clears it).
- **Stars and diary:** playing together 30 s / 1 min / 5 min gives 1/2/3 stars, logged in the diary (catalogue in the house); diary jams replay on the record player.
- **Gathering:** random stick trees with a sparkle; both players gather (E / O); the friend fetches too.
- **Fishing (rewritten):** cast at the dock end. Struct `Fishing` with `tries` (3), `hit_by` and `early` per player, `lesson`. Ring closes every `FISH_CUE_GAP` (8 steps); hit window 0.22 s, perfect 0.09 s. Co-op: either player's hit counts, both = double. 1P friend on the dock sometimes saves a miss. First cast shows an onboarding card (`fish_card`), a big E! prompt, and the lesson fish cannot escape; `fish_lesson` is saved as `lessons 1`.
- **Cooking:** percussion on the beat in the pot.
- **House size:** `Home::size` 0-3 (saved as `room N`): 7x5, 9x6, 11x7, then the penthouse loft (`home.loft()`). Use `home.w()`, `home.d()`, `home.door()`, never fixed sizes. Loft: back `LOFT_D` rows from `LOFT_X0`, height `LOFT_H`; stairs in column `stair_x()` over `STAIR_LEN` rows. `home.level(x, z)` and `home.height_at(p)` give the level and height; `walkable` refuses positions whose corners differ in height by more than 0.35, which makes the loft edge a wall and the stairs a ramp. Players get `Player::lift`, furniture `home.piece_lift(piece)`. The upgrade is the first catalogue row (`Row::Upgrade`).
- **House:** room at `home::ORIGIN`, starting at 7x5 tiles, catalogue at (0,0) with acorns as currency. `home::Piece` has `item, x, z, rot, state`; `state` stores lamp/light/speaker/subwoofer off (1), plant growth (0-3), and the dish on the table (index+1). Saved as `item:x:z:rot` or `item:x:z:rot:state`.
- **Furniture uses (E):** lamp and red light toggle the lighting; bed sleeps (fade, saves); sofa and concrete bench sit (`Player::perch` lifts the sitter); plant waters and blooms; bookshelf opens the reading view (see below); table serves cooked dishes; speaker/subwoofer toggle their effects (`house.switched_on`); steel pillar clangs; rug dance; piano and drums play; record player plays diary jams; wind chime rings.
- **DJ booth:** E enters first-person DJ mode (`dj[0]`), camera from `booth_pov`; pads 1-6, W/S tempo, A/D pattern, R record, X leave. Player 2 joins with O for b2b (`dj[1]`, pads 7-=). Two round turntables spin while the beat is on. The panel shows Serato decks in two columns (deck 1 left/red, deck 2 right/yellow) from `live_decks`.
- **Mic sync (F):** works natively (cpal) and in the browser (getUserMedia → ScriptProcessor ring buffer → `mic_pull` into `OnsetDetector`). The light is a single dot: blinking amber "syncing...", solid green "synced N bpm".
- **Pause (Esc or click):** mixer with five sliders, and a restart button (press twice, or R twice) that saves and then reloads the page or relaunches the app.
- **Per-player clear:** C clears all in 1P; in 2P C clears player 1's notes and Y clears player 2's.
- **Radio:** SoundCloud/Mixcloud widgets in the page, tracklists, optional BPM that the beat follows.
- **Mobile:** floating translucent Joy-Con style buttons, fullscreen in portrait and landscape; button labels and states come from `ui_report`.

## The Serato bridge

`python3 bridge/serato_live.py` (run from the project folder):

1. Reads Serato DJ Pro 4's SQLite library (`~/Library/Application Support/Serato/Library/master.sqlite`, table `history_entry`, rows with `end_time = -1` are loaded decks), or older `_Serato_/History/Sessions/*.session` files.
2. Serves `web/` on http://localhost:4747 and answers `/now` with `{track, source, decks: [{deck, track}]}`; the page polls it when opened on localhost.
3. Posts each change to ntfy.sh (topic in `web/sets.js`) so GitHub Pages viewers see it; `--offline` turns that off, `--debug` prints what it can see.

The game published as a Claude artifact cannot receive track names (the page is not allowed to reach the laptop or other sites); only localhost and GitHub Pages can.

## Save format

Plain text lines, one keyword each: `tiles`, `inv`, `camp`, `dishes`, `cook`, `chars`, `spots`, `sound`, `notes`, `songbook`, `partner`, `mix`, `acorns`, `lessons`, `home`, `style`, `diary ...`. Saved every 5 seconds, on quit, on sleep and before restart. Desktop: `~/.melody-quack/save.txt`; browser: localStorage key `melody-quack-save`. When saving inside the house, the position stored is the door outside.

## Testing that has been used

- Native: build only (no display in the sandbox).
- Web: a local static server plus Playwright with Chromium (`--use-gl=swiftshader`), seeding localStorage with a save, sending key presses, and taking screenshots. A fake microphone stream can be injected by overriding `navigator.mediaDevices.getUserMedia`.
- The tempo estimator was checked offline on a synthetic 124 bpm kick (locks at 123.8 bpm in about 4 seconds).

## Not verified / open items

- Real Serato on the owner's Mac worked for the newest track; the two-deck view was tested only with a fake database.
- Sound quality, iPhone feel, and a real fishing catch with human timing.
- Only player 1 can use furniture; player 2 can only join the DJ booth.
- Roadmap not started: online friend invite (2 players over the network) and an online music party.
- The owner also asked to be taught how to build all of this in Rust, one lesson at a time.

## Lighting and sleeping (latest)

- `Light` has a `lamps` list: every switched-on lamp and red light in the house adds its own glow (`Light::apply`), so all lights are equally bright. `Batch::unlit` draws a box without shading, used for glowing lamp shades and the red light bar.
- Sleeping: player 1 lies on the left half of the bed, player 2 on the right; in 1-player mode the friend joins on the right and gets up when you do. `Player::draw` lays the body flat with its head on the pillow and draws a quilt over it.
- The sixth note is C (MIDI 84); it used to be labelled B#.

## Fishdex and pixel emoji (latest)

- `src/fish.rs`: 120 species (`SPECIES`), `rarity`, `acorns`, `kind` (maps to the 3 cooking categories in `inv.fish`), `pick` (weighted by rarity, `luck` from perfect hits), `search` (number or name), and `art` (procedural 18x11 pixel picture from name keywords). Catch counts are `dex_counts`, saved as `dex n,n,...`.
- The fishdex is a modal like the pause menu (`dex_open`, drawn before the pause block, `continue`s the frame), so typing never reaches game keys. Phones search with `window.prompt` through the `ask_text_open`/`ask_text_get` imports.
- `src/emoji.rs`: 12 8x8 pixel emojis and the friend's `reply`. `Player::emote` draws a bubble of voxel pixels tilted toward the camera. Keys: Z for player 1 (GraveAccent or Apostrophe also work; the web maps backquote to Apostrophe), the inventory bag moved to Enter, Period or Slash for player 2 (the web cannot read Slash). Touch buttons: emoji `data-key="KeyZ"`, bag `data-key="Enter"`.

## Speaker recognition and suggestions (latest)

- `bridge/listen.py`: `Listener` runs in a daemon thread with its own asyncio loop. Every `--listen-every` seconds (20) it records `--listen-seconds` (8) of mono 16-bit audio from `--device`, calls `Shazam().recognize`, and on a new track fills `heard` and `next` (from `related_tracks`, then the track's `relatedtracksurl`, then `similar_from_deezer`: Deezer `search` → `artist/{id}/related` → each related artist's `artist/{id}/top?limit=1`, no key needed). On the owner's Mac Shazam recognised songs but returned no related tracks, which is why the Deezer fallback exists. `listener.where` says which source answered, `listener.problem` and `listener.note()` give Terminal messages (quiet mic under loudness 300, no match, errors). Each song is a dict `{artist, title, track, shazam, soundcloud, artist_soundcloud}`; the SoundCloud links are plain search URLs, because the SoundCloud API needs a registered app and has no documented related-tracks endpoint. `heard` clears after 3 minutes without a match.
- `shazamio` needs Python 3.10+; macOS ships 3.9. The import is lazy in `start_listener`, so the bridge still runs without it (it prints the install hint). `--no-listen` disables it; `--list-devices` lists inputs.
- `/now` now also carries `heard` and `next`. With no Serato history, the bridge keeps running if the listener started, and the heard track becomes `track` with source "heard on your speakers".
- Page: `takeLive` sets `window.melodyHeard` and `window.melodyNext` (track names, one per line) and fills `#now-heard` with the artist link, SoundCloud and Shazam links and a "goes well with" list.
- Game: `radio::next()` (imports `radio_next_len`/`radio_next_get`, native stub empty) is polled into `live_next` with `live_decks`; the DJ booth panel draws up to four lines under "goes well with". The page puts the heard track first in `melodyNext` with a leading `=`; Rust draws it as a blue `heard:` line (hidden when it already is the now-playing line).
- Tested with fake `shazamio` and `sounddevice` modules on `PYTHONPATH` (Shazam cannot be reached from the sandbox); not yet tested with a real microphone and real Shazam.

## DJ set mode and karaoke (latest)

- Flags live in the page: `flags = {djset, karaoke}` saved in localStorage `melody-live-flags`. Wasm imports `live_flag_get(i)` / `live_flag_set(i, on)` (0 = djset, 1 = karaoke; native stub is an `AtomicU8`). The pause menu has two buttons under fishdex/restart (keys J and K); the page has `#toggle-djset` and `#toggle-karaoke`.
- `render()` decides what shows: djset off → `liveText` is the heard track and `melodyDecks` is empty; djset on → Serato track and decks. The raw values are `rawSerato`, `rawDecks`.
- Karaoke: `listen.py` reads `matches[0].offset` from the Shazam result and stores `sync = {pos, stamp}` (pos = seconds into the song at `stamp`, the epoch time the clip started recording), refreshed on every recognition. On a new song `find_lyrics` searches LRCLIB (`/api/search`, track+artist, then title without brackets, then free text) and `parse_lrc` turns `syncedLyrics` into `[[seconds, text], ...]`. `/now` carries `lyrics` and `sync`; both are stripped before posting to ntfy (size limit).
- The overlay `#karaoke` is a fixed-position HTML element over the canvas (HTML, because the game's default font cannot draw Korean or Japanese). `karaokeTick` runs every animation frame: position = `sync.pos + now - sync.stamp`, current line with a `--fill` gradient, next line below, a `k-song` tag when djset is off. The game calls `live_overlay(y)` every frame with the vertical position as a fraction of the canvas (0.52 normally, 0.075 in the DJ booth, 0 to hide while paused, in the fishdex or the character picker).

## Bookshelf reading view and catalogue fix (latest)

- `src/books.rs` holds `BOOKS` (5) and `SPREADS` (5 spreads of 2 pages). Content is factual; check facts before adding pages.
- Reading is a modal like the fishdex: `shelf_open`, `shelf_sel`, `open_book: Option<usize>`, `book_spread`, `page_turn` (1 → 0 animation), `turn_dir`. It runs before the fishdex block and `continue`s the frame, and draws straight to the screen at native resolution (not the low-res target) so the text stays sharp.
- Text is painted into render targets with `Camera2D` (`paint_page` 512x680 per page, `paint_spine` 640x96 per book), only when the spread changes (`painted`), and mapped onto 3D quads with `draw_affine_parallelogram` (the second edge vector is the texture's horizontal axis). The scene boxes still use `Batch` with `reading_light()`. `draw_shelf_scene` and `draw_book_scene` hold the layouts; the turning page is an untextured quad rotating around the spine.
- Keys: A/D (or J/L, arrows) choose and turn, E/O/Enter/Space open or turn forward, X/Esc/Backspace back. Touch: tap thirds of the screen; an x button top right.
- The catalogue preview texture is drawn with `flip_y: false` (it was upside down with `true`, unlike the main world target).

## Mushrooms, real-time sky, tabs, fishing tweaks (latest)

- **Clock:** `mod clock` (pair): `now()`, `offset()` (web: `Date`; native: `SystemTime` and `date +%z`), `weather()` (-1 unknown, 0 clear, 1 cloudy, 2 rain, 3 snow), `temp()`, `sun(0|1)` (sunrise/sunset unix), `lat()`. Web imports `sky_now/offset/weather/temp/sun/lat`; the page fills `window.melodySky` from Open-Meteo (geolocation, else Hamburg 53.55/9.99) every 20 minutes. The Claude artifact cannot fetch, so `weather()` stays -1 there and the weather is simulated from the season and a seeded roll per day and 3-hour block.
- **Sky:** `update_sky()` runs once per frame (where `dusk` is computed) and stores `Sky {night, glow, hour, day, season, weather, temp}` in the `SKY` thread-local; read it with `sky()`. `night` uses the circular distance to sunrise/sunset (ramps over about an hour), `glow` peaks at sunset/sunrise. `dusk` targets the real night (a jam only adds a little). `sky_haze` tints fog and `Light::apply` adds sunset warmth and weather dimming. `render_view` draws the sky band (sunset gradient, sun/moon, clouds, stars only when clear) and rain/snow as boxes around the camera. `draw_world` picks grass/leaf colours by season and turns white when snowing. The house light darkens with `night`.
- **Mushrooms:** `shrooms: Vec<(Vec2, usize)>` (max 10), `shroom_today` from `mushroom::today(sky.day)`, `shroom_clock` (14 s, 7 s in rain; 6 at once when the day starts). Spawned next to tree tiles; drawn by `draw_shrooms` from the `SHROOMS` thread-local. Picking happens in the outdoor E handler before gathering. Saved as `mush a b c` (basket by kind) and `shroomdex n,...`. The pop card is `fish_pop` with a fourth `bool` = mushroom.
- **Dishes:** `DISHES` has 8 (grilled mushrooms, mushroom stew, fish & mushroom hotpot, golden truffle soup added); recipe priority is in the cooking block. `stars_for`: 3 stars at 180 s.
- **Dex modal:** `dex_mode` (0 fish, 1 mushrooms) and `dex_from_shop`. Tabs in the header; Left/Right/Tab or clicking switch; catalogue/diary reopen the shop when the dex came from it. Shop W/S goes up to tab 3 (mushrooms).
- **Trees:** `walkable` only blocks the trunk (`TRUNK_HALF` 0.12 around the tile centre).
- **Fishing:** `Fishing::tight` (0.72-0.92 after the lesson, rerolled each ring) scales the hit and perfect windows; the next ring comes 6, 8 or 10 steps later. "perfect!/nice!/together!" draw small above the ring using `BOBBER_AT` (projected in `render_view` with the world camera matrix).
- **Books:** six (cook & quack added); pages and spines are repainted every frame while reading, because the font atlas can be incomplete on the first paint; the reading camera looks from almost straight above.
- **Artifact page:** the fragment builder (outside the repo) appends CSS that hides the header, footer text and radio section and makes the canvas fill the viewport on black.

## Arcade, tv, mirror ball, deeper loft (latest)

- **Items** appended at the end of `home::ITEMS` (indices 17-19, save compatible): `MirrorBall` (300, non-solid, `state` 0 = on), `Arcade` (500, solid 1x1, its dance pad is drawn on the tile in front), `Tv` (1000, non-solid 2x1, hangs at the back of its tiles, `state` 0 = off, 1-5 = channel).
- **Audio bands:** `tempo::OnsetDetector` also splits the mic into four bands (one-pole lowpasses at 150/600/3000 Hz) and keeps `bands` (smoothed levels) and `hits` (band onsets with a per-band refractory time). The main loop copies them into the `VIS` thread-local (`Vis {bands, pulse, pads}`) each frame; without a listening mic the bands are made from the beat phase and the players' key glow.
- **Tv:** `tv_pixel(channel, x, y, w, h, time, vis)` returns the colour of each cell of a 24x14 grid of unlit boxes (`TV_CHANNELS`).
- **Dance arcade:** `Ddr {on, arrows, score, combo, judge, flash, last_mic}`; `ddr_request` is set by the furniture handler. Starting it places the player on the pad, turns the mic on (`listening`), blocks walking (`walk_dir`) and skips the furniture handler for that player. Mic band hits map to lanes via `DDR_LANE_OF_BAND`, are quantised to eighth notes and scheduled one bar (16 steps) later; when the mic has been quiet for 3 s the step-crossing block schedules arrows from `BEAT_STYLES` and looper notes instead, and after 4 s with no beat and no loop the beat is switched on. `Ddr::add` dedupes per lane and slot and caps two arrows per slot. Judging windows 0.05/0.1/0.18 s. Esc stops it without opening pause. The overlay is drawn with `draw_pixel_grid` and `ddr_arrow_pixels`.
- **Help:** `help_box()` gives the panel rect; three pages (`help_page`), arrows or clicks switch, Esc/H/click outside close (`help_closed` stops Esc from also pausing).
- **Deeper loft:** `ROOM_SIZES` has five entries, `MAX_SIZE` 4, `LOFT_SIZE` 3, `loft_depth(size)` / `Home::loft_d()` (2 rows, 4 at size 4). Every former `LOFT_D` use goes through `loft_d()`; `upgrade_blocker` checks the target size.
- **Penthouse window:** drawn in `draw_home` from `sky()`: unlit sky bands mixed between day, night, sunset and grey weather, sun or moon and stars, clouds, harbour water, building facades with lit windows (more at night), two cranes with blinking lights, and rain or snow in front.
- **Catalogue preview:** render target 400x150, preview camera `fovy` 0.62 rad (it was 36, an accidental huge value that had been flipping the picture), drawn with `flip_y: true`.
- **Gathering priority:** the outdoor E handler checks mushrooms (0.9 / 0.8 radius), then sticks lying next to stick trees (`stick_spot`), before `gather_at` shakes a tree.

## Bag storage, quests, plants, sampler, picker preview (latest)

- **Bag:** `stored: Vec<home::Item>` (saved as `stored i,i,...`). M or Enter (player 2: Backspace) toggles the bag; with a piece in hand inside, the key stores it instead. `bag_busy` (bag open inside) blocks walking and the furniture handler for `bag_owner` and routes W/S, E (take out into `holding`) and X twice (`bag_clear_armed`, pack everything) to the bag. M no longer toggles music mode (Tab does).
- **Upgrades:** `Home::in_the_way()` replaces `upgrade_blocker`: pieces on the future stairs, across the future loft edge, or solid on the tile at the top of the stairs are moved into `stored` before `size += 1`.
- **Quests:** `Quests {day, list}`, `daily_quests(day, has_plant, has_arcade)` picks 3 of `QUEST_KINDS` from a hash of the day; `quest_event(&mut quests, kind, amount, &mut inv, &mut toasts)` is called from the catch, mushroom, cooking, watering, stick, shake, note, dance, star and emoji code. Saved as `quests day k:t:p:d,...` and restored only for the same day.
- **Plants:** `Piece::day` (days since 1970 when last watered, saved as an optional 6th field `item:x:z:rot:state:day`). Each frame a plant with `today - day >= 2` becomes `home::WITHERED` (state 4); watering revives it, growth only happens on the first watering of a day.
- **Sampler:** `Item::Sampler` (index 20, 180 acorns). `Sampler {open, owner, grid [[bool;16];4], cursor, pads [Option<Sound>;4], recording, step}`; the grid is saved as `seq a b c d` (16-bit masks). Steps play from the looper step-crossing block while inside. Built-in rows use the generated kick/clap/hat/voice through `play_ch`; recorded pads use `Audio::play_private`, which skips the tape and jam logs, so mic recordings never reach .wav exports. Recording uses `OnsetDetector::capture` (1.5 s at the mic rate) and `tidy_recording` (trim, resample to 44.1 kHz, normalise, fade); pads live only in memory.
- **Picker preview:** `render_pick_preview` draws both chosen animals on plinths into `pick_view` (300x130); the selected one turns and sings.
- **Emoji:** heart highlight moved into the left lobe, the sleepy z is black, the music note heads have a gap. The concrete preset is now `Home::techno_club` and the catalogue row is "techno club set".

## Since this handoff: the split, the PC bang and the chapel

- **`main.rs` is now setup plus a short loop.** Items moved, unchanged, into topic files (`world.rs`, `music.rs`, `voices.rs`, `critters.rs`, `home_draw.rs`, `save.rs`, `sky.rs`, `ui.rs` and others); platform pairs live in `src/native/` and `src/web/`.
- **State:** everything kept between frames is in `Game` (`state.rs`, the loop's `g`); values worked out each frame are in `Frame` (`frame_state.rs`, the loop's `fr`).
- **`src/steps/`** holds one function per phase of a frame, each taking `(&mut Game, &mut Frame)`, called in order from `main`. `bookshelf`, `fishdex`, `pause_menu` and `venues::frame` take over the whole frame.
- **Venues (`venues.rs`, `steps/venues.rs`):** the PC bang (tiles `PCBANG`) and St. Bassilica (tiles `CHAPEL`) are stamped onto the map by `venues::stamp`, also after loading an old save. Interiors sit at `PC_ORIGIN` and `CH_ORIGIN`, far off the map like the home. `VenueState` is `g.venue`; it is not saved.
- **PC bang:** sit at a computer (E) for the Quack 95 desktop: quacknet (a hidden `.` link opens the underground flyer page), rave.exe (books a rave, gives a code and a flyer), readme, and a terminal with a small capture the flag (`ls -a`, `su goose` with the tempo 132 as password, `rot13`) that reveals the secret rave's code.
- **Chapel:** the goose at the door takes a code typed in or picked from codes already seen; old flyers' codes are refused. Inside, `rave_tick` sequences techno from the existing drum, bass, bell, marimba and piano sounds, seeded by the rave. If the radio reports a live track, the generated set stays silent and the panel shows the live title.
- **Icons:** `assets/icon16/32/64.png` (window) and `web/icon64/180/192/512.png` (page and manifest) are scaled from the owner's icon artwork.

## Online play

- **`server/`** is its own Cargo package (not part of the game's build): a websocket relay with rooms keyed by rave code. Text lines only. Client to server: `host <code> <sound|bpm|seed|title>`, `find <code>`, `join <code> <character>`, `at <x> <z> <fx> <fz> <walking>`, `sing <note>`, `leave`. Server to client: `room <code> <age ms> <info>`, `none <code>`, `peer <id> <character> <x> <z> <fx> <fz> <walking>`, `sing <id> <note>`, `left <id>`. Empty rooms are dropped after six hours.
- **`net` module pair:** `src/native/net.rs` (tungstenite on a thread) and `src/web/net.rs`. Web imports added to the plugin in `web/index.html`: `net_open`, `net_send`, `net_state`, `net_recv_len/get`, `net_server_len/get`. The address comes from `MELODY_LIVE.server` in `web/sets.js`, or `~/.melody-quack/server.txt` / `MELODY_QUACK_SERVER` on desktop.
- **In the game:** `online()` in `steps/venues.rs` runs every frame: connects and retries every ten seconds, announces hosted raves, answers the goose's lookups, keeps `g.venue.peers`, and sends player one's position twelve times a second while in the chapel. The room's age from the server sets `rave_start`, which is what keeps the set in time for everyone.
- **Limits:** only the chapel is shared; the island, the PC bang and the home are not. Only player one is sent. There are no names or chat, and no accounts: anyone with a code can come in.

## The name, the post and the timed list

- **Name:** `g.venue.name`, asked by `name_screen` in `steps/venues.rs` before the character picker whenever it is empty. Saved as a new `name <text>` line; older saves load and are asked once.
- **Server event (`Event` in `server/src/main.rs`):** one scheduled secret rave at a time. `mark` is the start minus 30 minutes. `sit` records one attempt per token within 10 s of the mark; `settle` ranks attempts by absolute offset and keeps the best 30; `claim` gives the code to those 30 at once and to up to 70 others from the mark plus five minutes. `join` and `find` on the secret code answer `early` before the start, and any room answers `full` at 100 players. The rules are unit tested with a fake clock (`cargo test` in `server/`).
- **Extra messages:** client `hello <token>`, `sit`, `claim`, `join <code> <players> <ch0> <ch1> <name>`, `at <slot> ...`, `sing <slot> <note>`; server `event <ms to start> <phase> <mine> <rank> <general left> <info>` twice a second, `sat <offset ms>`, `code <code>`, `nocode early|list|full <ms>`, `early <code> <ms>`, `full <code>`, `who <id> <players> <ch0> <ch1> <name>`, `peer <id> <slot> ...`, `sing <id> <slot> <note>`.
- **In the game:** `g.venue.tonight` holds the latest event; `mail_icon` draws the envelope (called at the end of a normal frame and from the venue HUDs), `mail` is the announcement overlay. The token is random per launch, so a place on the list does not survive restarting the game.
- **Timing is judged on the server** when the `sit` message arrives, so a slow connection counts against the player.

## DJ membership

- Server (`server/src/main.rs`): `dj <key>` signs in as a DJ (from the `--djs` file, `name = key` lines, reloaded on each sign-in) or as the admin (`--admin`). `dj -` signs out. `ask` makes a request for the room the DJ is in; admins get `request <id> <code> <name>`. `grant <id>` / `deny <id>` answer it; a grant is kept per (dj, code), is used up by `golive`, and adds 1 to the DJ's credit (`--credits` file). `settle <name>` clears a credit. `golive decks` or `golive audio <https url>` puts the DJ on air in their room (`live <id> <mode> <url|-> <name>`, `live 0` when it ends); `mix <beat> <melody> <sound> <bpm>` and `pad <i>` are relayed to the room only from the DJ who is on air.
- Client: the state is in `VenueState` (`dj_key`, `dj_name`, `credit`, `admin`, `requests`, `balances`, `granted`, `on_air`, `mix`, `bpm_now`). `steps/venues.rs`: `name_screen` is also the welcome screen after restart (`welcome` flag) and has the "become a dj" part; `decks` is the booth panel (`Overlay::Decks`); the admin page is in `mail`; `rave_tick` follows `mix` while a DJ is live on the decks and plays nothing while one is live with real audio.
- `net::open_url`, `net::listen`, `net::can_listen` (web: `net_open_url`, `net_listen` in `web/index.html`). On the web the stream plays in an audio element; on desktop the link is opened in the browser.
- Save: lines `djkey <key>` and `welcome 1`, appended.
- Tested with a local server: first-run key entry, ask, grant from a script and from the admin page in the game, credit going up and being cleared, going live on the in-game decks with pads and mix reaching a second client, the welcome screen after restart. Not tested: real audio links, the web build of these screens, anything with a real Patreon page.
- Looks: the DJ on air sends `look <0-3>` (`LOOKS` in `venues.rs`); the server keeps it with the live set and sends it to late joiners. `chapel` picks floor, wall and light colours from it and `backdrop` paints the scene on the back wall. The top bar is slimmer and the code tag moves to the bottom left while a look is on. Tested on desktop with a second client receiving the look; not tested in the browser.
