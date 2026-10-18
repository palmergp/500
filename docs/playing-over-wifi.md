# Playing with friends over wifi

One computer hosts the table; everyone else joins from the browser on their phone, tablet or laptop.
Everyone needs to be on the same wifi. Nothing goes over the internet.

## Start a table (on the host computer)

1. Double-click **`play.cmd`** in the project folder, or run `play` in a terminal there.
   It builds whatever has changed, starts the host and opens the game in your browser.
   The first run takes a few minutes; after that it starts in seconds.
2. The window shows a QR code and an address like `http://192.168.1.158:5500/`.
3. The first time, Windows asks whether to let `fh-host` through the firewall. Allow it on **private networks**,
   or friends won't be able to connect.

Leave the window open while you play. Press **Ctrl+C** to stop.

## Join (everyone, including the host)

- On the host computer, open `http://localhost:5500/`.
- On a phone, scan the QR code, or type the address it shows into the browser.

The **Table** screen opens.

1. Type your name.
2. Press **Sit here** on a seat. North and South are partners, and so are East and West.
3. Choose a bot for each empty seat, and choose the pace.
4. Anyone sitting at the table can press **Deal**.

## During the game

- You see only your own cards, and you always sit at the bottom of your screen.
- **Hint**, grades and hand reviews work as in solo play. The host computer works them out from what your seat
  can see, and nobody else sees your hints. Undo is turned off, since other people are playing too.
- After each hand, the next deal waits until everyone has pressed **Next hand**.
- If a phone sleeps or drops off the wifi, the game waits for that player.
  - When they reopen the page, they get their seat and cards back.
  - If they've gone, anyone can press **Let a bot play for them**.
- **Table** (top right) shows the seats.
  - Someone new can take over a bot's seat, which lets a late arrival join a game already going.
  - You can leave, and a bot takes your seat.
  - **End game…** ends the game for everyone.
- The game is saved on the host (in `fh-host-table.json`). Stopping and restarting the host picks it up where you
  left off. To start clean, run it with `--fresh`.

## Options

Pass them to the script (for example `play --fresh`). `play --help` lists them, and `--no-browser` skips
opening the browser.

| Option | What it does |
|---|---|
| `--port N` | Listen on a different port. The default is 5500. |
| `--dist DIR` | Serve the app from another folder. |
| `--state FILE` | Keep the saved game in another file. |
| `--fresh` | Ignore the saved game. |

## If it doesn't work

- **A phone can't open the address.** Check the firewall prompt above, and that the phone is on the same wifi,
  not mobile data or a guest network. Some guest and office networks block devices from reaching each other.
- **The address shows the wrong network.** This can happen with a VPN or virtual network adapters.
  Use the computer's wifi address instead (shown by `ipconfig`), with `:5500` on the end.
- **"Can't listen on port 5500".** The host is already running in another window, or another program is using
  the port. Try `--port 5501`.
- **Solo play still works on the host's address.** Press **Play solo instead** on the Table screen. To come back,
  press **Join table**.
