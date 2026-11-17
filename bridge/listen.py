import asyncio
import io
import re
import json
import threading
import time
import urllib.parse
import urllib.request
import wave
import warnings

RATE = 44100


def soundcloud_search(text):
    return "https://soundcloud.com/search?q=" + urllib.parse.quote(text)


def soundcloud_artist(name):
    return "https://soundcloud.com/search/people?q=" + urllib.parse.quote(name)


def song(artist, title, url=""):
    artist, title = (artist or "").strip(), (title or "").strip()
    both = f"{artist} - {title}" if artist and title else (title or artist)
    return {
        "artist": artist,
        "title": title,
        "track": both,
        "shazam": url or "",
        "soundcloud": soundcloud_search(both),
        "artist_soundcloud": soundcloud_artist(artist) if artist else "",
    }


def deezer(path, **query):
    url = "https://api.deezer.com/" + path + ("?" + urllib.parse.urlencode(query) if query else "")
    request = urllib.request.Request(url, headers={"User-Agent": "melody-quack"})
    with urllib.request.urlopen(request, timeout=10) as reply:
        data = json.loads(reply.read().decode("utf-8"))
    if isinstance(data, dict) and data.get("error"):
        raise RuntimeError(data["error"].get("message", "deezer error"))
    return data


def similar_from_deezer(artist, title, count=6):
    found = deezer("search", q=f'artist:"{artist}" track:"{title}"', limit=1).get("data") or deezer("search", q=f"{artist} {title}", limit=1).get("data") or deezer("search/artist", q=artist, limit=1).get("data")
    if not found:
        return []
    first = found[0]
    artist_id = first["artist"]["id"] if "artist" in first else first["id"]
    picks = []
    for other in (deezer(f"artist/{artist_id}/related", limit=count * 2).get("data") or [])[: count * 2]:
        try:
            top = deezer(f"artist/{other['id']}/top", limit=1).get("data") or []
        except Exception:
            continue
        if top:
            picks.append(song(other.get("name"), top[0].get("title"), top[0].get("link", "")))
        if len(picks) >= count:
            break
    return picks


def secure():
    import ssl
    try:
        import certifi
        return ssl.create_default_context(cafile=certifi.where())
    except ImportError:
        return ssl.create_default_context()


def lrclib(**query):
    url = "https://lrclib.net/api/search?" + urllib.parse.urlencode(query)
    request = urllib.request.Request(url, headers={"User-Agent": "melody-quack (https://github.com)"})
    with urllib.request.urlopen(request, timeout=10, context=secure()) as reply:
        return json.loads(reply.read().decode("utf-8"))


def parse_lrc(text):
    lines = []
    for raw in (text or "").splitlines():
        stamps = re.findall(r"\[(\d+):(\d+(?:\.\d+)?)\]", raw)
        words = re.sub(r"\[[^\]]*\]", "", raw).strip()
        for minutes, seconds in stamps:
            lines.append([round(int(minutes) * 60 + float(seconds), 2), words])
    lines.sort(key=lambda line: line[0])
    return lines


def find_lyrics(artist, title):
    plain_title = re.sub(r"\s*[\(\[].*?[\)\]]", "", title).strip() or title
    main_artist = re.split(r"\s*(?:,|&| x | feat\.?| ft\.?| with )\s*", artist, flags=re.I)[0].strip() or artist
    queries = [
        {"track_name": title, "artist_name": artist},
        {"track_name": plain_title, "artist_name": main_artist},
        {"q": f"{main_artist} {plain_title}"},
        {"track_name": plain_title},
    ]
    problem, plain_only = "", False
    for query in queries:
        try:
            found = lrclib(**query)
        except Exception as error:
            problem = str(getattr(error, "reason", error))
            continue
        for item in found if isinstance(found, list) else []:
            if item.get("syncedLyrics"):
                lines = parse_lrc(item["syncedLyrics"])
                if lines:
                    return lines, ""
            if item.get("plainLyrics"):
                plain_only = True
    if problem and not plain_only:
        if "CERTIFICATE" in problem.upper():
            return [], "could not reach the lyrics site (certificates). run: pip install certifi"
        return [], f"could not reach the lyrics site: {problem}"
    if plain_only:
        return [], "this song only has lyrics without timing, so they can't follow along"
    return [], "no timed lyrics for this song yet"


def list_devices():
    import sounddevice

    for index, device in enumerate(sounddevice.query_devices()):
        if device["max_input_channels"] > 0:
            print(f"  {index}: {device['name']}")


class Listener:
    def __init__(self, seconds=8.0, every=20.0, device=None, related=6):
        import sounddevice
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", RuntimeWarning)
            from shazamio import Shazam

        self.sd = sounddevice
        self.shazam = Shazam()
        self.seconds = seconds
        self.every = every
        self.device = int(device) if device is not None and str(device).isdigit() else device
        self.related = related
        self.heard = None
        self.next = []
        self.heard_at = 0.0
        self.status = "starting"
        self.loudness = 0
        self.where = ""
        self.lyrics = []
        self.lyrics_note = ""
        self.sync = None
        self.clip_started = 0.0
        self.problem = ""
        self.changed = threading.Event()

    def start(self):
        threading.Thread(target=lambda: asyncio.run(self.run()), daemon=True).start()

    def record(self):
        frames = int(RATE * self.seconds)
        self.clip_started = time.time()
        with self.sd.RawInputStream(samplerate=RATE, channels=1, dtype="int16", device=self.device) as stream:
            data, _ = stream.read(frames)
        raw = bytes(data)
        self.loudness = max((abs(int.from_bytes(raw[i:i + 2], "little", signed=True)) for i in range(0, len(raw) - 1, 400)), default=0)
        out = io.BytesIO()
        with wave.open(out, "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(RATE)
            w.writeframes(raw)
        return out.getvalue()

    def note(self):
        if self.status == "quiet":
            return "the microphone hears almost nothing. turn the speakers up, allow Terminal in System Settings > Privacy & Security > Microphone, or pick another input with --device (see --list-devices)"
        if self.status == "no match":
            return "heard sound but Shazam did not know the song (it may be unreleased, or too quiet or noisy)"
        if self.status.startswith("error"):
            return f"listening hit a problem: {self.status[7:]}"
        if self.status == "recognising":
            return "heard music, asking Shazam..."
        return ""

    async def related_to(self, track):
        found = []
        key = track.get("key")
        try:
            if key and str(key).isdigit():
                data = await self.shazam.related_tracks(track_id=int(key), limit=self.related)
                found = data.get("tracks", []) if isinstance(data, dict) else []
        except Exception:
            found = []
        if not found and track.get("relatedtracksurl"):
            try:
                data = await self.shazam.http_client.request("GET", track["relatedtracksurl"], headers=self.shazam.headers())
                found = data.get("tracks", []) if isinstance(data, dict) else []
            except Exception:
                found = []
        picks = [song(t.get("subtitle"), t.get("title"), t.get("url")) for t in found[: self.related] if t.get("title")]
        if picks:
            self.where = "Shazam"
            return picks
        try:
            picks = await asyncio.to_thread(similar_from_deezer, track.get("subtitle") or "", track.get("title") or "", self.related)
            self.where = "Deezer"
        except Exception as error:
            self.problem = f"could not find tracks that go with it: {error}"
            picks = []
        return picks

    async def run(self):
        while True:
            started = time.time()
            try:
                self.status = "listening"
                clip = await asyncio.to_thread(self.record)
                if self.loudness < 300:
                    self.status = "quiet"
                    await asyncio.sleep(max(1.0, self.every - (time.time() - started)))
                    continue
                self.status = "recognising"
                result = await self.shazam.recognize(clip)
                track = result.get("track") if isinstance(result, dict) else None
                if track and track.get("title"):
                    heard = song(track.get("subtitle"), track.get("title"), track.get("url"))
                    started_at = self.clip_started
                    matches = result.get("matches") or []
                    offset = matches[0].get("offset") if matches and isinstance(matches[0], dict) else None
                    if not self.heard or heard["track"] != self.heard["track"]:
                        picks = await self.related_to(track)
                        try:
                            words, why = await asyncio.to_thread(find_lyrics, heard["artist"], heard["title"])
                        except Exception as error:
                            words, why = [], f"lyrics search failed: {error}"
                        self.sync = None
                        self.heard, self.next, self.lyrics, self.lyrics_note = heard, picks, words, why
                        self.changed.set()
                    if isinstance(offset, (int, float)):
                        self.sync = {"pos": round(float(offset), 2), "stamp": round(started_at, 2)}
                    self.heard_at = time.time()
                    self.status = "heard"
                else:
                    self.status = "no match"
                    if self.heard and time.time() - self.heard_at > 180:
                        self.heard = None
                        self.next = []
                        self.lyrics = []
                        self.lyrics_note = ""
                        self.sync = None
                        self.changed.set()
            except Exception as error:
                self.status = f"error: {error}"
            await asyncio.sleep(max(1.0, self.every - (time.time() - started)))
