import argparse
import functools
import http.server
import json
import os
import sqlite3
import struct
import sys
import threading
import time
import urllib.request
from pathlib import Path

DEFAULT_TOPIC = "melody-quack-live-b3c29ad39c3a"
GAME_FOLDER = Path(__file__).resolve().parent.parent / "web"


def serato_folder():
    return Path.home() / "Music" / "_Serato_"


def serato4_database():
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / "Serato" / "Library" / "master.sqlite"
    local = os.environ.get("LOCALAPPDATA")
    if local:
        return Path(local) / "Serato" / "Library" / "master.sqlite"
    return Path.home() / "AppData" / "Local" / "Serato" / "Library" / "master.sqlite"


def read_chunks(data, start=0, end=None):
    end = len(data) if end is None else end
    pos = start
    while pos + 8 <= end:
        tag = data[pos:pos + 4]
        size = struct.unpack(">I", data[pos + 4:pos + 8])[0]
        body = pos + 8
        yield tag, body, min(body + size, end)
        pos = body + size


def read_fields(data, start, end):
    fields = {}
    pos = start
    while pos + 8 <= end:
        field_id, size = struct.unpack(">II", data[pos:pos + 8])
        fields[field_id] = data[pos + 8:pos + 8 + size]
        pos += 8 + size
    return fields


def text(raw):
    if not raw:
        return ""
    return raw.decode("utf-16-be", errors="ignore").replace("\x00", "").strip()


def number(raw):
    return int.from_bytes(raw, "big") if raw and len(raw) <= 8 else 0


def label(artist, title):
    artist, title = (artist or "").strip(), (title or "").strip()
    if artist and title and artist.lower() not in title.lower():
        return f"{artist} - {title}"
    return title or artist


def session_entries(data):
    for tag, body, end in read_chunks(data):
        if tag != b"oent":
            continue
        for inner, ibody, iend in read_chunks(data, body, end):
            if inner == b"adat":
                yield read_fields(data, ibody, iend)


def from_session_file(folder):
    sessions = folder / "History" / "Sessions"
    files = [p for p in sessions.glob("*.session") if p.is_file()] if sessions.is_dir() else []
    if not files:
        return None
    newest = max(files, key=lambda p: p.stat().st_mtime)
    if time.time() - newest.stat().st_mtime > 6 * 3600:
        return None
    entries = list(session_entries(newest.read_bytes()))
    open_now = [f for f in entries if not number(f.get(29))]
    pool = open_now or entries
    pool = sorted(pool, key=lambda f: number(f.get(28)))
    for fields in reversed(pool):
        track = label(text(fields.get(7)), text(fields.get(6)))
        if track:
            return track
    return ""


class Serato4:
    def __init__(self, path):
        self.path = path
        self.columns = None

    def connect(self):
        uri = "file:" + urllib.request.pathname2url(str(self.path)) + "?mode=ro"
        db = sqlite3.connect(uri, uri=True, timeout=0.5)
        db.row_factory = sqlite3.Row
        return db

    def available(self):
        return self.path.is_file()

    def history_columns(self, db):
        if self.columns is None:
            self.columns = [row[1] for row in db.execute("PRAGMA table_info(history_entry)")]
        return self.columns

    def rows(self, db, active_only=True, limit=8):
        cols = self.history_columns(db)
        if not cols:
            return []
        where = "WHERE end_time = -1" if active_only and "end_time" in cols else ""
        order = "start_time" if "start_time" in cols else "id"
        return list(db.execute(f"SELECT * FROM history_entry {where} ORDER BY {order} DESC, id DESC LIMIT {int(limit)}"))

    def track_of(self, row):
        keys = row.keys()
        pick = lambda *names: next((str(row[n]) for n in names if n in keys and row[n] not in (None, "")), "")
        return label(pick("artist", "track_artist", "artist_name"), pick("title", "name", "track_name", "song"))

    def decks(self):
        db = self.connect()
        try:
            cols = self.history_columns(db)
            found = {}
            newest = ""
            for row in self.rows(db, limit=32):
                track = self.track_of(row)
                if not track:
                    continue
                newest = newest or track
                deck = row["deck"] if "deck" in cols and row["deck"] is not None else len(found) + 1
                found.setdefault(deck, track)
            return newest, sorted(found.items(), key=lambda item: str(item[0]))
        finally:
            db.close()

    def debug(self):
        db = self.connect()
        try:
            print("history_entry columns:", ", ".join(self.history_columns(db)))
            print("loaded right now (end_time = -1):")
            for row in self.rows(db):
                print("  ", {k: row[k] for k in row.keys()})
            print("newest rows overall:")
            for row in self.rows(db, active_only=False)[:3]:
                print("  ", {k: row[k] for k in row.keys()})
        finally:
            db.close()


def send(topic, message):
    request = urllib.request.Request(
        f"https://ntfy.sh/{topic}",
        data=message.encode("utf-8"),
        headers={"Title": "melody quack live", "Tags": "notes"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=10) as response:
        return response.status


class State:
    track = ""
    source = ""
    decks = []
    heard = None
    next = []
    lyrics = []
    sync = None


def payload():
    return {
        "track": State.track,
        "source": State.source,
        "decks": [{"deck": deck, "track": track} for deck, track in State.decks],
        "heard": State.heard,
        "next": State.next,
        "lyrics": State.lyrics,
        "sync": State.sync,
    }


def start_listener(args):
    if args.no_listen:
        return None
    try:
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from listen import Listener
        listener = Listener(seconds=args.listen_seconds, every=args.listen_every, device=args.device)
    except ImportError as error:
        version = f"{sys.version_info.major}.{sys.version_info.minor}"
        print("")
        print("*** listening to your speakers is OFF ***")
        if sys.version_info < (3, 10):
            print(f"this bridge was started with Python {version}, which is too old for Shazam.")
            print("start it with the newer Python instead:")
            print("  python3.12 bridge/serato_live.py")
        else:
            print(f"Python {version} is missing a helper ({error.name}). install the helpers with:")
            print(f"  python{version} -m pip install shazamio sounddevice")
        print("")
        return None
    except Exception as error:
        print(f"could not start listening: {error}")
        return None
    listener.start()
    print(f"listening to your speakers every {args.listen_every:.0f} seconds to recognise the track")
    return listener


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def guess_type(self, path):
        return "application/wasm" if str(path).endswith(".wasm") else super().guess_type(path)

    def do_GET(self):
        if self.path.startswith("/now"):
            body = json.dumps(payload()).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        super().do_GET()


def serve(port):
    handler = functools.partial(Handler, directory=str(GAME_FOLDER))
    try:
        server = http.server.ThreadingHTTPServer(("127.0.0.1", port), handler)
    except OSError as error:
        if error.errno in (48, 98, 10048):
            sys.exit(f"\nport {port} is already in use - the bridge is probably still running in another Terminal window.\nstop it there with ctrl+c (or type: lsof -ti :{port} | xargs kill), then start this again.")
        raise
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server


def main():
    parser = argparse.ArgumentParser(description="Show the track playing in Serato DJ inside Melody Quack.")
    parser.add_argument("topic", nargs="?", default=DEFAULT_TOPIC, help="the live channel name from web/sets.js")
    parser.add_argument("--serato", type=Path, default=serato_folder(), help="path to the _Serato_ folder (Serato 3 and older)")
    parser.add_argument("--database", type=Path, default=serato4_database(), help="path to master.sqlite (Serato 4)")
    parser.add_argument("--every", type=float, default=2.0, help="seconds between checks")
    parser.add_argument("--port", type=int, default=4747, help="port for the local game and track feed")
    parser.add_argument("--offline", action="store_true", help="only show the track in your local game, send nothing online")
    parser.add_argument("--test", action="store_true", help="only print what Serato is playing")
    parser.add_argument("--debug", action="store_true", help="print what the bridge can see in Serato's files, then stop")
    parser.add_argument("--no-listen", action="store_true", help="do not recognise what plays on your speakers")
    parser.add_argument("--listen-every", type=float, default=20.0, help="seconds between recognitions")
    parser.add_argument("--listen-seconds", type=float, default=8.0, help="seconds of sound to record each time")
    parser.add_argument("--device", default=None, help="input to listen on, by number or name (see --list-devices)")
    parser.add_argument("--list-devices", action="store_true", help="list the inputs you can listen on, then stop")
    args = parser.parse_args()

    if args.list_devices:
        try:
            sys.path.insert(0, str(Path(__file__).resolve().parent))
            from listen import list_devices
            list_devices()
        except ImportError:
            print("install sounddevice first: python3 -m pip install sounddevice")
        return

    serato4 = Serato4(args.database)
    has_old = (args.serato / "History").is_dir()
    has_serato = serato4.available() or has_old
    if not has_serato and (args.debug or args.no_listen):
        sys.exit(
            "could not find Serato history.\n"
            f"  looked for Serato 4 at {args.database}\n"
            f"  and Serato 3 at {args.serato / 'History'}\n"
            "pass --database or --serato with the right path"
        )

    if args.debug:
        if serato4.available():
            print(f"Serato 4 database: {args.database}")
            try:
                serato4.debug()
                for deck, track in serato4.decks()[1]:
                    print(f"  deck {deck}: {track}")
            except sqlite3.Error as error:
                print("  could not read it:", error)
        if has_old:
            print(f"Serato 3 history: {args.serato / 'History' / 'Sessions'}")
            print("  newest track there:", from_session_file(args.serato) or "(nothing in the last 6 hours)")
        return

    def check():
        if not has_serato:
            return "", "", []
        if serato4.available():
            try:
                track, decks = serato4.decks()
                if track:
                    return track, "serato 4", decks
            except sqlite3.Error:
                pass
        if has_old:
            track = from_session_file(args.serato)
            if track:
                return track, "serato history", []
        return "", "", []

    listener = start_listener(args)
    if not has_serato and not listener:
        sys.exit("found neither Serato history nor a way to listen - see the messages above")
    if not args.test:
        serve(args.port)
        print(f"open http://localhost:{args.port} to play with the track names on the decks")
        if not args.offline:
            print(f"also sending to listeners on the live channel {args.topic}")
    print("play some music. press ctrl+c to stop.")
    last = None
    said = ""
    try:
        while True:
            if listener and listener.problem:
                print(f"{time.strftime('%H:%M:%S')}  {listener.problem}")
                listener.problem = ""
            if listener and listener.status != said:
                said = listener.status
                note = listener.note()
                if note:
                    print(f"{time.strftime('%H:%M:%S')}  {note}")
            track, source, decks = check()
            heard = listener.heard if listener else None
            State.heard, State.next = heard, (listener.next if listener else [])
            State.lyrics = listener.lyrics if listener and heard else []
            State.sync = listener.sync if listener and heard else None
            if not track and heard:
                track, source = heard["track"], "heard on your speakers"
            State.track, State.source, State.decks = track, source, decks
            snapshot = (track, tuple(decks), heard["track"] if heard else "", tuple(n["track"] for n in State.next))
            if track and snapshot != last:
                last = snapshot
                stamp = time.strftime("%H:%M:%S")
                print(f"{stamp}  now playing: {track}  ({source})")
                for deck, name in decks:
                    print(f"          deck {deck}: {name}")
                if heard:
                    print(f"          heard: {heard['track']}")
                    if State.next:
                        print(f"          goes well with (from {listener.where}):")
                    for n in State.next:
                        print(f"            + {n['track']}")
                    print(f"          karaoke lyrics: {len(State.lyrics)} lines" if State.lyrics else "          karaoke lyrics: none found for this song")
                if not args.test and not args.offline:
                    try:
                        rich = len(decks) > 1 or bool(heard)
                        online = payload()
                        online.pop("lyrics", None)
                        online.pop("sync", None)
                        send(args.topic, json.dumps(online) if rich else track)
                    except OSError as error:
                        print(f"{stamp}  could not send online ({error}), will retry")
                        last = None
            time.sleep(args.every)
    except KeyboardInterrupt:
        if not args.test and not args.offline and last:
            try:
                send(args.topic, "__end__")
            except OSError:
                pass
        print("\nstopped")


if __name__ == "__main__":
    main()
