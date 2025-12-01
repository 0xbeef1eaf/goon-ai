// Simple test to show a prompt window
const handle = await goon.textPrompt.show({
    text: "Hello from Slint! Type this text to continue...",
    fontSize: 32,
    color: [1.0, 1.0, 1.0, 1.0],
    background: [0.0, 0.0, 0.0, 1.0],
    alignment: "center",
    window: {
        size: { width: 800, height: 600 },
        position: { x: 100, y: 100 }
    }
});

console.log("Prompt window created:", handle);
