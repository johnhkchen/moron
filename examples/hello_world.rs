//! Hello World — A minimal moron scene demonstrating the core API.
//!
//! Build: `moron build examples/hello_world.rs -o hello_world.mp4`

use moron::prelude::*;

struct HelloWorld;

impl Scene for HelloWorld {
    fn build(m: &mut M) {
        // Set theme and voice
        m.theme(Theme::default());
        m.voice(Voice::kokoro());

        // Title card
        m.title("Hello, World!");
        m.narrate("Welcome to moron, a motion graphics engine written in Rust.");
        m.play(FadeIn);

        m.beat();

        // Key points
        m.section("What is moron?");
        m.narrate("moron turns simple Rust scene files into professional explainer videos.");
        m.steps(&[
            "Write a scene file in Rust",
            "Run moron build",
            "Get a finished MP4",
        ]);
        m.play(Stagger(FadeUp.with_ease(Ease::OutBack)));

        m.breath();

        // Metric showcase
        m.narrate("And it does this with surprisingly little code.");
        m.metric("Lines of Code", "< 15K", Direction::Down);
        m.play(CountUp);

        m.beat();

        // Closing
        m.narrate("All running offline, on your local machine.");
        m.show("Offline. Fast. Professional.");
        m.play(FadeIn);
    }
}
