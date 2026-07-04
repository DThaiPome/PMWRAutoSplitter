use asr::{Address, Error, Process, watcher::Watcher};

const OFFSET_LOADING_FLAG: u64 = 0x126488;
const OFFSET_LOCATION_ID: u64 = 0x218518;
const OFFSET_LAPS_COMPLETED: u64 = 0x126608;
const OFFSET_TOTAL_LAPS: u64 = 0x1264C8;
const OFFSET_TIME_ELAPSED: u64 = 0x217A74;

#[derive(Default)]
pub struct Memory {
    process_base_address: Address,
    pub loading_flag: Watcher<bool>,
    pub location_id: Watcher<i32>,
    pub laps_completed: Watcher<i32>,
    pub total_laps: Watcher<i32>,
    pub time_elapsed: Watcher<i32>
}

impl Memory {
    pub fn init(process: &Process) -> Option<Self> {
        match process.get_module_address("PMR.exe") {
            Ok(x) => {
                asr::print_message("Initialized memory watchers");
                let mut memory = Memory::default();
                memory.process_base_address = x;
                return Some(memory);
            },
            Err(_) => {
                asr::print_message("Failed to init process memory watchers!");
                return None;
            },
        }
    }

    pub fn update(&mut self, process: &Process) {
        update_bool(process, self.address_of(OFFSET_LOADING_FLAG), &mut self.loading_flag);
        update_int(process, self.address_of(OFFSET_LOCATION_ID), &mut self.location_id);
        update_int(process, self.address_of(OFFSET_LAPS_COMPLETED), &mut self.laps_completed);
        update_int(process, self.address_of(OFFSET_TOTAL_LAPS), &mut self.total_laps);
        update_int(process, self.address_of(OFFSET_TIME_ELAPSED), &mut self.time_elapsed);
    }

    fn address_of(&self, offset: u64) -> Address {
        return self.process_base_address.add(offset);
    }

    pub fn current<T>(watcher: Watcher<T>) -> T 
    where T: Default, {
        return watcher.pair.unwrap_or_default().current;
    }

    pub fn old<T>(watcher: Watcher<T>) -> T 
    where T: Default, {
        return watcher.pair.unwrap_or_default().old;
    }
}

fn update_bool(process: &Process, address: Address, watcher: &mut Watcher<bool>) {
    let current_value = watcher.pair.unwrap_or_default().current;
    let result: Result<i32, Error> = process.read(address);
    if result.is_err() {
        asr::print_message("BOOL ERROR");
        return;
    };
    let val = result.map_or(current_value, |v| v == 1);
    watcher.update_infallible(val);
}

fn update_int(process: &Process, address: Address, watcher: &mut Watcher<i32>) {
    let current_value = watcher.pair.unwrap_or_default().current;
    let val = process.read(address).unwrap_or(current_value);
    watcher.update_infallible(val);
}