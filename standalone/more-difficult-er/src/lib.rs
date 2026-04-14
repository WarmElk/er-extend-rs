mod config;

use std::time::Duration;
use eldenring::cs::{CSEventFlagMan, CSTaskGroupIndex, CSTaskImp, ClearCountCorrectParam, SoloParamRepository, WorldChrMan};
use eldenring::fd4::FD4TaskData;
use eldenring::param::CLEAR_COUNT_CORRECT_PARAM_ST;
use eldenring::util::system::wait_for_system_init;
use er_extend_rs_esd::initialize_er_extend_rs_esd_from_config;
use er_extend_rs_rva::HookError;
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};
use tracing_subscriber::filter::LevelFilter;
use crate::config::MoreDifficultERConfig;

trait FlagManExtender {
    fn compare_and_set_flag(&mut self, flag: u32, expected: bool, set_to: bool) -> bool;
    fn get_flag(&self, flag: u32) -> bool;
    fn set_flag(&mut self, flag: u32, set_to: bool);
}

impl FlagManExtender for CSEventFlagMan {
    fn compare_and_set_flag(&mut self, flag: u32, expected: bool, set_to: bool) -> bool {
        let matches = self.virtual_memory_flag.get_flag(flag) == expected;
        if matches {
            self.virtual_memory_flag.set_flag(flag, set_to);
        }
        matches
    }

    fn get_flag(&self, flag: u32) -> bool {
        self.virtual_memory_flag.get_flag(flag)
    }

    fn set_flag(&mut self, flag: u32, set_to: bool) {
        self.virtual_memory_flag.set_flag(flag, set_to)
    }
}

struct MoreDifficultER {
    current_difficulty_level: Option<u32>,
    config: Option<MoreDifficultERConfig>,
    hooking_error: Option<HookError>,
    original_data: Vec<MoreDifficultData>,
}

struct MoreDifficultData {
    max_hp_rate: f32,
    physics_attack_rate: f32,
    magic_attack_rate: f32,
    fire_attack_rate: f32,
    thunder_attack_rate: f32,
    dark_attack_rate: f32,
}

impl MoreDifficultData {
    fn update_with_raw_original_data(&self, data: &mut CLEAR_COUNT_CORRECT_PARAM_ST) {
        data.set_max_hp_rate(self.max_hp_rate);
        data.set_physics_attack_rate(self.physics_attack_rate);
        data.set_magic_attack_rate(self.magic_attack_rate);
        data.set_fire_attack_rate(self.fire_attack_rate);
        data.set_thunder_attack_rate(self.thunder_attack_rate);
        data.set_dark_attack_rate(self.dark_attack_rate);
    }

    fn update_with_original_data_for_multiplier(&self, data: &mut CLEAR_COUNT_CORRECT_PARAM_ST, multiplier: f32) {
        data.set_max_hp_rate(self.max_hp_rate() * multiplier);
        data.set_physics_attack_rate(self.physics_attack_rate() * multiplier);
        data.set_magic_attack_rate(self.magic_attack_rate() * multiplier);
        data.set_fire_attack_rate(self.fire_attack_rate() * multiplier);
        data.set_thunder_attack_rate(self.thunder_attack_rate() * multiplier);
        data.set_dark_attack_rate(self.dark_attack_rate() * multiplier);
    }

    fn max_hp_rate(&self) -> f32 {
        self.fix_zero(self.max_hp_rate)
    }

    fn physics_attack_rate(&self) -> f32 {
        self.fix_zero(self.physics_attack_rate)
    }

    fn magic_attack_rate(&self) -> f32 {
        self.fix_zero(self.magic_attack_rate)
    }

    fn fire_attack_rate(&self) -> f32 {
        self.fix_zero(self.fire_attack_rate)
    }

    fn thunder_attack_rate(&self) -> f32 {
        self.fix_zero(self.thunder_attack_rate)
    }

    fn dark_attack_rate(&self) -> f32 {
        self.fix_zero(self.dark_attack_rate)
    }

    fn fix_zero(&self, value: f32) -> f32 {
        match value {
            0.0 => 1.0,
            f => f
        }

    }
}

impl MoreDifficultER {
    fn new() -> Self {
        Self {
            current_difficulty_level: None,
            config: None,
            hooking_error: None,
            original_data: vec![],
        }
    }

    fn init(&mut self) {
        let config = config::get_config();

        let logging_level = if config.log_debug_messages() { LevelFilter::DEBUG } else { LevelFilter::WARN };

        tracing_subscriber::fmt::fmt()
            .with_max_level(logging_level)
            .compact()
            .init();

        tracing::debug!("Config: {:?}", config);

        self.config = Some(config);
        self.initialize_esd_config();

    }

    fn initialize_esd_config(&mut self) {
        if let Some(ref config) = self.config &&
            let Err(hook_error) = initialize_er_extend_rs_esd_from_config(&config.extra_config) {

            tracing::error!("Failed to initialize additional grace menu hook: {:?}", hook_error);
            self.hooking_error = Some(hook_error);
        }
    }

    fn step(&mut self) {
        let Some(_) = unsafe { WorldChrMan::instance() }.ok() else {
            return;
        };
        let Some(flag_man) = unsafe { CSEventFlagMan::instance() }.ok() else {
            return;
        };

        if self.original_data.is_empty() {
            self.original_data = initialize_more_difficult_er_data();
        }

        const BASE_SAVED_DIFFICULTY_FLAG_ID: u32 = 1061460000;
        const MAX_DIFFICULTY_LEVEL: u32 = 5;

        self.handle_more_difficult_er_menu(flag_man, BASE_SAVED_DIFFICULTY_FLAG_ID, MAX_DIFFICULTY_LEVEL);
        self.handle_more_difficult_er_difficulty(flag_man, BASE_SAVED_DIFFICULTY_FLAG_ID, MAX_DIFFICULTY_LEVEL);
    }

    fn handle_more_difficult_er_menu(&self, flag_man: &mut CSEventFlagMan, base_saved_difficulty_flag_id: u32, max_difficulty: u32) {
        const BASE_DIFFICULTY_FLAG_ID: u32 = 1061472100;

        (0..=max_difficulty).for_each(|level| {
            let flag_id = BASE_DIFFICULTY_FLAG_ID + level;
            if flag_man.compare_and_set_flag(flag_id, true, false) {
                for saved_level in 0..=max_difficulty {
                    let level_to_set = saved_level == level;
                    flag_man.compare_and_set_flag(base_saved_difficulty_flag_id + saved_level, !level_to_set, level_to_set);
                    tracing::debug!("Set saved difficulty flag for level {} to {}", base_saved_difficulty_flag_id + saved_level, level_to_set);
                }
            }
        });

        let nothing_set = (0..=max_difficulty).all(|saved_level| {
            !flag_man.get_flag(base_saved_difficulty_flag_id + saved_level)
        });
        if nothing_set {
            flag_man.set_flag(base_saved_difficulty_flag_id, true);
        }
    }

    fn handle_more_difficult_er_difficulty(&mut self, flag_man: &mut CSEventFlagMan, base_saved_difficulty_flag_id: u32, max_difficulty: u32) {
        let Some(repo) = unsafe { SoloParamRepository::instance() }.ok() else {
            tracing::error!("SoloParamRepository instance not found, cannot handle more difficult ER difficulty");
            return;
        };
        if let Some(difficulty_level_flag) =
            (0..=max_difficulty)
                .find(|level_flag| flag_man.get_flag(*level_flag + base_saved_difficulty_flag_id))
                .filter(|level| level != &self.current_difficulty_level.unwrap_or(u32::MAX)){
            if difficulty_level_flag == 0 {
                (0..=7)
                    .for_each(|ng_plus| {
                        if let Some(ng_plus_data) = repo.get_mut::<ClearCountCorrectParam>(ng_plus) &&
                            let Some(original_data) = self.original_data.get(ng_plus as usize) {
                            original_data.update_with_raw_original_data(ng_plus_data);
                        }
                    });
            }
            else {
                let multiplier = self.more_difficult_er_multiplier().powf(difficulty_level_flag as f32);
                tracing::debug!("Applying multiplier {} for difficulty level {}", multiplier, difficulty_level_flag);
                (0..=7)
                    .for_each(|ng_plus| {
                        if let Some(ng_plus_data) = repo.get_mut::<ClearCountCorrectParam>(ng_plus) &&
                            let Some(original_data) = self.original_data.get(ng_plus as usize) {
                            original_data.update_with_original_data_for_multiplier(ng_plus_data, multiplier);
                        }
                    });
            }
            self.current_difficulty_level = Some(difficulty_level_flag);
        }
    }

    fn more_difficult_er_multiplier(&self) -> f32 {
        const DEFAULT_MULTIPLIER: f32 = 1.1;
        match &self.config {
            None => DEFAULT_MULTIPLIER,
            Some(config) => config.more_difficult_er_multiplier.unwrap_or(DEFAULT_MULTIPLIER),
        }
    }
}

fn initialize_more_difficult_er_data() -> Vec<MoreDifficultData> {
    let Some(repo) = unsafe { SoloParamRepository::instance() }.ok() else {
        tracing::error!("SoloParamRepository instance not found, cannot initialize more difficult ER difficulty");
        return vec![];
    };
    (0..=7)
        .filter_map(|ng_plus| repo.get::<ClearCountCorrectParam>(ng_plus))
        .map(|ng_plus_data| MoreDifficultData {
            max_hp_rate: ng_plus_data.max_hp_rate(),
            physics_attack_rate: ng_plus_data.physics_attack_rate(),
            magic_attack_rate: ng_plus_data.magic_attack_rate(),
            fire_attack_rate: ng_plus_data.fire_attack_rate(),
            thunder_attack_rate: ng_plus_data.thunder_attack_rate(),
            dark_attack_rate: ng_plus_data.dark_attack_rate(),
        })
        .collect()
}

#[unsafe(no_mangle)]
/// # Safety
/// This is exposed this way such that libraryloader can call it. Do not call this yourself.
pub unsafe extern "C" fn DllMain(_hmodule: u64, reason: u32) -> bool {
    // Exit early if we're not attaching a DLL
    if reason != 1 {
        return true;
    }

    std::thread::spawn(move || {
        wait_for_system_init(&Program::current(), Duration::MAX).expect("Timeout waiting for system init");

        let mut more_difficult_er = MoreDifficultER::new();

        more_difficult_er.init();

        let cs_task = unsafe { CSTaskImp::instance().unwrap() };
        cs_task.run_recurring(
            move |_: &FD4TaskData| {
                more_difficult_er.step();
            },
            CSTaskGroupIndex::FrameBegin,
        );
    });

    true
}
