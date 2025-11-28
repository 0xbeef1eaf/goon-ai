# goon.ai

An LLM-controlled background application that spawns interactive GUI elements through TypeScript execution in a sandboxed runtime.

## Overview

goon.ai is a Rust-based application that allows Language Models to control desktop media experiences through a type-safe TypeScript API. The LLM generates TypeScript code that compiles and executes in a secure sandbox, triggering GUI windows, media playback, and system interactions based on mood-aware asset selection.

## Key Features

- **LLM-Driven Interface**: LLMs generate TypeScript code to control the application
- **Type-Safe Execution**: TypeScript compilation catches errors before execution
- **Mood-Based Asset Selection**: Assets automatically selected based on current mood and optional tags
- **Permission System**: Granular control over which capabilities are enabled
- **Rich Media Support**: Images, videos, audio, animated GIFs, text prompts
- **Handle-Based Control**: Created windows and media can be controlled after creation
- **Modular SDK**: Only permitted APIs are exposed to the LLM
- **Minimal Context**: Asset lists never sent to LLM, reducing token usage by ~68%

## Architecture

```
┌─────────────┐
│     LLM     │ Generates TypeScript based on SDK
│  (Ollama)   │ (No knowledge of specific assets)
└──────┬──────┘
       │ TypeScript: await image.show({ tags: ['beach'] })
       ▼
┌─────────────┐
│ SWC Compiler│ Validates and compiles TypeScript
└──────┬──────┘
       │ JavaScript
       ▼
┌─────────────┐
│ deno_core   │ Executes in sandboxed runtime
│  Runtime    │ Calls Rust ops
└──────┬──────┘
       │ op_show_image()
       ▼
┌─────────────┐
│ Rust Ops    │ Select asset by mood + tags
│   Layer     │ Render to screen
└─────────────┘
```

## Core Concepts

### Packs

Packs are content bundles containing:
- Assets (images, videos, audio, GIFs, wallpapers)
- Mood definitions with associated tags
- LLM configuration and prompts
- Website links
- Permission requirements

### Moods

Moods are contextual states that filter asset selection:
- Each mood has associated tags
- Assets are filtered by mood tags at runtime
- LLM can query and change moods
- Current mood affects all asset operations

### Permissions

Fine-grained permission system:
- `image` - Display images and animations
- `video` - Play video files
- `audio` - Play audio files
- `prompt` - Show text prompts
- `wallpaper` - Set desktop wallpaper
- `website` - Open URLs in browser

SDK modules are only generated for granted permissions.

### Asset Selection

Assets are never listed in LLM prompts:
1. LLM calls `image.show({ tags: ['beach', 'sunny'] })`
2. Rust layer gets current mood (e.g., "Nature")
3. Filter assets by mood tags (`nature`, `landscape`, `outdoor`)
4. Further filter by optional tags (`beach` AND `sunny`)
5. Randomly select from filtered set
6. Fallback to mood-only if no matches

This reduces context size from ~7500 to ~2350 tokens.

## TypeScript SDK

The LLM receives a modular, type-safe SDK:

```typescript
// Image display with handle control
const img = await image.show({
    tags: ['beach', 'sunny'],
    position: { x: 100, y: 100, width: 800, height: 600 },
    opacity: 0.9,
    timeout: 10000
});

await img.setOpacity(0.5);
await img.moveTo(200, 200);
await img.close();

// Video playback
const vid = await video.play({
    tags: ['nature', 'calming'],
    loop: true,
    volume: 0.7
});

// Audio playback
const audio_handle = await audio.play({
    tags: ['ambient'],
    volume: 0.5,
    loop: true
});

// Text prompt with optional image
await textPrompt.show(
    "Take a moment to breathe...",
    { tags: ['calming'] }
);

// Mood management
const mood = await pack.getCurrentMood();
await pack.setMood("Nature");

// System operations
await wallpaper.set(['mountain', 'sunset']);
await website.open(['conservation']);
```

## Application Flow

1. **Initialize**: Load settings, pack, and assets
2. **Generate SDK**: Create TypeScript definitions based on permissions
3. **Main Loop**:
   - Build LLM context (system prompt + mood + SDK, NO assets)
   - Call LLM for TypeScript code
   - Compile with SWC (retry on error)
   - Execute in deno_core (retry on error)
   - Rust ops select assets and perform actions
   - Feed results back to LLM
   - Repeat

## Technology Stack

- **Rust**: Core application and runtime
- **ollama-rs**: LLM integration (local Ollama server)
- **SWC**: TypeScript compilation
- **deno_core**: JavaScript execution sandbox
- **winit + wgpu**: Window management and GPU rendering
- **libmpv2**: Video playback
- **rodio**: Audio playback
- **cosmic-text**: Text rendering

## Project Status

🚧 **In Active Development** 🚧

This project is currently in the planning and early implementation phase. See the [GitHub Issues](https://github.com/0xbeef1eaf/goon.ai/issues) for detailed implementation tasks and progress.

### Implementation Roadmap

- [ ] Core initialization and configuration loading (#2)
- [ ] Pack asset loading with mood-based selection (#3)
- [ ] LLM integration via ollama-rs (#4)
- [ ] TypeScript compilation pipeline (#5)
- [ ] deno_core runtime and ops (#6)
- [ ] SDK generation system (#7)
- [ ] Permission system (#8)
- [ ] GUI window manager with winit/wgpu (#9)
- [ ] Media rendering (images, video, audio, text) (#10-13)
- [ ] System integration (wallpaper, websites) (#14-15)
- [ ] Main application loop (#16)
- [ ] Testing infrastructure (#17)
- [ ] Documentation (#18)

## Getting Started

### Prerequisites

- Rust (edition 2024)
- Ollama server running locally
- Supported OS: Linux, Windows 10/11, macOS

### Installation

```bash
# Clone the repository
git clone https://github.com/0xbeef1eaf/goon.ai.git
cd goon.ai

# Build the project
cargo build --release

# Copy example settings
cp settings.example.yaml settings.yaml

# Edit settings.yaml with your preferences
$EDITOR settings.yaml

# Run the application
cargo run --release
```

### Configuration

**settings.yaml**:
```yaml
user:
  name: Your Name
  dob: 1990-01-01
  gender: male

llmSettings:
  host: "http://localhost:11434"  # Ollama server

runtime:
  permissions:
    - image
    - video
    - audio
    - prompt
    - wallpaper
    - website
  pack:
    current: Test Pack
    mood: Nature
```

**Pack Structure**:
```
packs/
  YourPack/
    config.yaml          # Pack configuration
    image/               # Image assets
    video/               # Video assets
    audio/               # Audio files
    hypno/               # GIF animations
    wallpaper/           # Wallpaper images
```

## Creating a Pack

See the `TestPack` for an example. A pack includes:

1. **config.yaml**: Metadata, LLM settings, asset definitions, moods
2. **Assets**: Organized by type with tags
3. **Moods**: Define tag filters for different contexts
4. **Permissions**: Declare required capabilities

Example `config.yaml`:
```yaml
meta:
  name: My Pack
  version: 1.0.0
  permissions:
    - image
    - audio

moods:
  - name: Relaxation
    description: Calming content for relaxation
    tags:
      - calm
      - peaceful
      - nature

assets:
  image:
    - path: image/beach.jpg
      tags:
        - beach
        - calm
        - sunny
```

## Safety and Security

- **Sandboxed Execution**: JavaScript runs in isolated deno_core runtime
- **Permission System**: Explicit permission grants required
- **Type Safety**: TypeScript compilation catches errors before execution
- **No File/Network Access**: LLM cannot access filesystem or network directly
- **Asset Validation**: All assets validated on pack load
- **Retry Limits**: Maximum 3 retries per error to prevent loops

## Contributing

Contributions are welcome! Please see the [Contributing Guide](CONTRIBUTING.md) for details.

1. Check [open issues](https://github.com/0xbeef1eaf/goon.ai/issues)
2. Fork the repository
3. Create a feature branch
4. Make your changes with tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust and the amazing Rust ecosystem
- Inspired by the potential of LLM-controlled interfaces
- Thanks to the Ollama team for local LLM inference

## Links

- [Documentation](https://github.com/0xbeef1eaf/goon.ai/wiki)
- [Issue Tracker](https://github.com/0xbeef1eaf/goon.ai/issues)
- [Architecture Documentation](AGENTS.md)

---

**Note**: This is experimental software. Use at your own risk. The LLM-generated code runs in a sandbox, but always review pack permissions before granting them.
