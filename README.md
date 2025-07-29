# clash

**Cards (Ludicrous Ones) Against Humanity (CLASH)** is an open‑source spin on Cards Against Humanity. Built in Rust and Svelte for offering a lightning‑fast online playground.

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
cargo run -- localhost:8080 -f http://localhost:5173 --cache data/cache --cert data/cert/cert.pem --key data/cert/key.pem
```

> The backend is also self-hosted at: [https://api.clash.nwrenger.dev/](https://api.clash.nwrenger.dev/)

**Command-line arguments**:

| Argument | Description                                           | Default                                                      |
| -------- | ----------------------------------------------------- | ------------------------------------------------------------ |
| `<HOST>` | **Required**. Socket address for the server (IP:port) | _None_                                                       |
| `-f`     | Allowed CORS origin for the frontend                  | `https://api.clash.nwrenger.dev`                             |
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

## Architecture

### [Backend](./) (Rust)

- **Endpoints:**
  - `[POST] /lobby` — Creates a new lobby and returns its UUID.
  - `[ANY] /ws/:lobby_id` — WebSocket endpoint to join and interact with a lobby.
- **Core:** Game state managed in-memory, clients communicate via WebSockets.
- **Server:** Runs on a central host, handling broadcasting and private messages.

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

## Contributing & Issues

CLASH is in **active beta**. We welcome:

- Bug reports
- Feature requests
- Pull requests

Please open issues or PRs on [GitHub](https://github.com/nwrenger/clash/issues).

## License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) for details.
