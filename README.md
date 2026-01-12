# clash

**Cards (Ludicrous Ones) Against Humanity (clash)** is an open‑source spin on Cards Against Humanity. Built in Rust and Svelte for offering a lightning‑fast online playground.

## Project Structure

### [Backend](./) (Rust)

- **Endpoints:**
  - `[POST] /lobby` — Creates a new lobby and returns its UUID.
  - `[ANY] /ws/:lobby_id` — WebSocket endpoint to join and interact with a lobby.
- **Core:** Game state managed in-memory, clients communicate via WebSockets.
- **Server:** Hosted centrally, responsible for message broadcasting and managing gameplay.

### [Frontend](./view) (Svelte + Skeleton)

- **Framework:** Svelte with Skeleton UI components.
- **Features:**
  - Create or join lobbies
  - General game loop with game over screen
  - Allowing custom decks from [clrtd](https://cast.clrtd.com/)
  - Real‑time updates via WebSockets
  - Responsive design for desktop and mobile
- **Deployment:**
  - Hosted on GitHub Pages
  - Automatically rebuilt and published on every commit

## Performance & Scaling

**clash** has been tested to handle up to **2–4 lobbies** with **5,000 players each**, or **10–30 lobbies** with **1,000 players each**, on typical modern server hardware ([Oracle's Ampere A1 in free tier](https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm)).

> ⚠️ **Note:** For very large lobbies (2,000+ players), browser performance becomes the primary bottleneck. Joining or updating lobbies with thousands of players can lag or freeze the UI, especially on lower-end devices.
>
> Actual capacity will vary depending on server CPU/RAM, network bandwidth, and client performance.

I recommend keeping lobby sizes below 5,000 for the best experience.

## Getting Started

### Prerequisites

- **Rust** (for the backend)
- **Node.js** & **bun** (for the frontend)

### Installation

1. **Clone the repo**

```bash
git clone https://github.com/nwrenger/clash.git
cd clash
```

2. **Backend** (Rust)

```bash
# Firstly generate some certificates
./data/cert/gen.sh
# Run Debug Build with frontend build
cargo run -- localhost:8080 -f http://localhost:5173 -c data/cache --cert data/cert/cert.pem --key data/cert/key.pem
```

> The backend is also self-hosted at: [https://api.clash.nwrenger.dev/](https://api.clash.nwrenger.dev/)

**Command-line arguments**:

| Argument | Description                                           | Default                                                      |
| -------- | ----------------------------------------------------- | ------------------------------------------------------------ |
| `<HOST>` | **Required**. Socket address for the server (IP:port) | _None_                                                       |
| `-f`     | Allowed CORS origin for the frontend                  | `https://clash.nwrenger.dev`                                 |
| `-c`     | Filesystem path where decks are stored                | `cache`                                                      |
| `--cert` | Path to the SSL certificate (`fullchain.pem`)         | `/etc/letsencrypt/live/api.clash.nwrenger.dev/fullchain.pem` |
| `--key`  | Path to the SSL private key (`privkey.pem`)           | `/etc/letsencrypt/live/api.clash.nwrenger.dev/privkey.pem`   |
| `--help` | Print help                                            | _None_                                                       |

3. **Frontend** (Svelte + Skeleton)

```bash
cd view
bun install
bun run dev
```

> The frontend is also hosted on GitHub Pages: [https://clash.nwrenger.dev/](https://clash.nwrenger.dev/)

## Contributing & Issues

Although **clash** is out of **beta**, I still welcome:

- Bug reports
- Feature requests
- Pull requests

Please open issues or PRs on [GitHub](https://github.com/nwrenger/clash/issues).

## License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) for details.
