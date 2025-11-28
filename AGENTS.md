# Technical Architecture & Agents

This document details the technical architecture of **goon.ai**, an LLM-controlled background application. It covers the system design, core components, data flow, and security model.

## System Architecture

goon.ai operates as a bridge between a local Large Language Model (LLM) and the user's desktop environment. The core application is written in Rust, which manages the lifecycle of a sandboxed JavaScript runtime (`deno_core`) and communicates with an Ollama server.

### High-Level Design

```mermaid
graph TB
    subgraph "Core Application"
        Main[Main Loop]
        Config[Configuration System]
        Pack[Pack Loader]
    end

    subgraph "LLM Layer"
        Ollama[Ollama Client]
        Prompt[Prompt Builder]
        Conv[Conversation Manager]
    end

    subgraph "Compilation & Execution"
        SWC[SWC Compiler]
        Deno[deno_core Runtime]
        SDK[SDK Generator]
    end

    subgraph "Permissions"
        PermResolver[Permission Resolver]
        PermChecker[Permission Checker]
    end

    subgraph "Asset Management"
        AssetRegistry[Asset Registry]
        MoodFilter[Mood-Based Filter]
        AssetSelector[Asset Selector]
    end

    subgraph "GUI & Rendering"
        WinMgr[Window Manager (winit + wgpu)]
        ImgRender[Image Renderer]
        VidRender[Video Renderer (libmpv2)]
        AudioPlayer[Audio Player]
        TextRender[Text Renderer]
    end

    Main --> Ollama
    Main --> SWC
    Main --> Deno
    
    Ollama --> Prompt
    Ollama --> Conv
    
    Config --> Pack
    Config --> PermResolver
    Pack --> AssetRegistry
    
    PermResolver --> PermChecker
    PermResolver --> SDK
    
    SDK --> Prompt
    SWC --> Deno
    
    Deno --> WinMgr
    Deno --> AudioPlayer
    Deno --> AssetSelector
    
    AssetSelector --> MoodFilter
```

## Core Components

### 1. LLM Integration (`src/llm/`)

The LLM layer is responsible for communicating with the local Ollama server. It constructs prompts that include the current system state, mood, and available SDK definitions, but notably **excludes** specific asset lists to conserve context window.

*   **Client**: Manages HTTP communication with the Ollama API.
*   **Prompt Builder**: Assembles the system prompt, mood context, SDK definitions, and conversation history.
*   **Conversation Manager**: Tracks the dialogue history and manages context truncation.

**Flow:**
1.  **Build Prompt**: System prompt + Mood + SDK Definitions + History.
2.  **Send**: Request sent to Ollama.
3.  **Receive**: TypeScript code response.
4.  **Parse**: Extract code block for compilation.

### 2. Runtime Environment (`src/runtime/`)

The runtime executes the LLM-generated code in a secure sandbox using `deno_core`.

*   **JsRuntime**: A V8-based JavaScript runtime initialized with necessary extensions.
*   **Ops**: Rust functions exposed to JavaScript (e.g., `op_show_image`, `op_play_audio`).
*   **Executor**: Coordinates the execution of compiled code.

**Key Ops:**
*   `op_show_image`: Display an image window.
*   `op_show_video`: Play a video.
*   `op_play_audio`: Play audio.
*   `op_set_wallpaper`: Change desktop wallpaper.
*   `op_get_asset`: Query assets by tag (internal use).

### 3. SDK Generation (`src/sdk/`)

To ensure type safety and hallucination-free code generation, the system dynamically generates a TypeScript SDK based on the active permissions.

*   **Generator**: Creates TypeScript module definitions.
*   **Modular Exports**: Only generates modules for granted permissions (e.g., if `video` permission is missing, the `video` module is not generated).
*   **Handle-Based Control**: APIs return handles (e.g., `ImageHandle`) allowing the LLM to manipulate created objects (move, close, change opacity) after creation.

**Structure:**
*   `index.ts`: Exports enabled modules.
*   `image.ts`, `video.ts`, `audio.ts`, etc.: Type definitions and classes for specific capabilities.

### 4. Permission System (`src/permissions/`)

Security is enforced through a granular permission system that intersects pack requirements with user grants.

*   **Permission Types**: `Image`, `Video`, `Audio`, `Hypno`, `Wallpaper`, `Prompt`, `Website`.
*   **Resolver**: Computes the effective permission set by checking what the Pack requests against what the User has allowed in `settings.yaml`.
*   **Checker**: Runtime enforcement. Every Rust op checks the `OpState` for the required permission before execution.

### 5. Asset Management

Assets are organized into "Packs" and filtered by "Moods".

*   **Packs**: Bundles of content (images, videos, config).
*   **Moods**: Contextual states (e.g., "Relaxation", "Focus") that define tag filters.
*   **Selection Logic**:
    1.  LLM requests an asset with optional tags (e.g., `['beach']`).
    2.  System applies current Mood filters (e.g., `['calm', 'nature']`).
    3.  System intersects with LLM tags.
    4.  Random selection from the result set.

## Data Flow

1.  **Initialization**: Load `settings.yaml`, load Pack `config.yaml`, resolve permissions.
2.  **SDK Gen**: Generate TypeScript definitions based on resolved permissions.
3.  **Loop**:
    *   Construct Prompt (System + Mood + SDK + History).
    *   **LLM** generates TypeScript.
    *   **SWC** compiles TypeScript to JavaScript.
    *   **Deno** executes JavaScript.
    *   **Rust Ops** intercept calls (e.g., `image.show()`).
    *   **Asset System** selects content.
    *   **GUI** renders content.
    *   Execution result returned to LLM context.

## Security Model

*   **Sandboxing**: Code runs in `deno_core`, isolated from the host system.
*   **No Filesystem Access**: The LLM cannot read/write arbitrary files.
*   **Explicit Permissions**: Capabilities must be explicitly granted by the user.
*   **Asset Validation**: Assets are validated upon pack loading.

## Tool Use

When interacting with package management files, always utilise the tool to ensure the best results.
Editing Cargo.toml or Cargo.lock directly is bad behaviour, always use command line `cargo ...`