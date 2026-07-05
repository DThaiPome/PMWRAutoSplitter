#![no_std]

mod settings;
mod memory;
mod splitter;

use asr::{Process, future::{next_tick, retry}, settings::Gui};
use settings::Settings;
use memory::Memory;

use crate::splitter::Splitter;

asr::async_main!(stable);
asr::panic_handler!();

struct ComponentState {
    ticks: u64,
    settings: Settings,
    memory: Memory,
    splitter: Splitter
}

const TICK_RATE: f64 = 120.0;

async fn main() {
    // TODO: Set up some general state and settings.
    let mut state: ComponentState = ComponentState {
        ticks: 0,
        settings: Settings::register(),
        memory: Memory::default(),
        splitter: Splitter::default()
    };
    asr::set_tick_rate(TICK_RATE);
    asr::print_message("PMWR Autosplitter Loaded");
    loop {
        let process = Process::wait_attach("PMR.exe").await;
        let memory = retry(|| Memory::init(&process)).await;
        state.memory = memory;
        process
            .until_closes(async {
                asr::print_message("PMWR Autosplitter Enabled (process found)");
                loop {
                    component_update(&process, &mut state);
                    state.ticks += 1;
                    next_tick().await;
                }
            })
            .await;
        asr::print_message("PMWR Autosplitter Disabled (process ended)");
        next_tick().await;
    }

}

fn component_update(process: &Process, state: &mut ComponentState) {
    state.settings.update();
    state.memory.update(process);
    state.splitter.update(state.ticks, &state.memory, &state.settings);
}