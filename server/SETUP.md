# Putting the rave server online

The game works offline, but chat, the DJ decks for everyone, the timed secret rave, sign-in and cloud saves need this server running on the internet.

## Fly.io (recommended)

Fly.io runs the server from the Dockerfile in this folder, gives it an https address and keeps a small disk for accounts, saves, the DJ list and credit. It costs a few euros a month and needs a card on the account.

1. Make an account at fly.io and install `flyctl` (on a Mac: `brew install flyctl`), then `fly auth login`.
2. In `server/fly.toml`, change `melody-quack-rave` (in `app` and in `MQ_PUBLIC_URL`) to a name nobody else has taken. Pick a `primary_region` near you (`ams` is Amsterdam, `fra` Frankfurt, `lhr` London).
3. From the `server` folder:
   ```
   fly launch --no-deploy --copy-config
   fly volumes create mq_data --size 1
   fly secrets set MQ_ADMIN='your admin password' MQ_ADMIN_NAME='mercurial9000' MQ_PATREON='https://www.patreon.com/...'
   fly deploy
   ```
4. Add DJs: `fly ssh console`, then `echo 'cayennepfefa = mq-...' >> /data/djs.txt`. The server reads the file each time a DJ signs in.
5. Optional sign-in keys: `fly secrets set MQ_GOOGLE_ID=... MQ_GOOGLE_SECRET=...` (see the README for Google and Apple). Steam sign-in works without keys.

The game address is `wss://<your app name>.fly.dev`.

## Point the games at it

- **Everyone's apps:** in the GitHub repository, Settings, Secrets and variables, Actions, Variables, add `MQ_SERVER` = `wss://<your app name>.fly.dev`. New builds (the .dmg, Linux and Arch packages, and the web version on GitHub Pages) then connect by themselves.
- **One computer, right away:** put the address in `~/.melody-quack/server.txt`, or start the game with `MELODY_QUACK_SERVER=wss://...`.

## Anywhere else

Any host that runs a Docker image and passes websockets works (Railway, Render's paid plan, a small VPS with Caddy in front). Keep it always on: the timed rave runs on the server's clock.
