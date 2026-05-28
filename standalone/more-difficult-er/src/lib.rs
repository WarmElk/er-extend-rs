mod config;

use std::collections::HashMap;
use std::time::Duration;
use eldenring::cs::{CSEventFlagMan, CSTaskGroupIndex, CSTaskImp, SoloParamRepository, SpEffectParam, WorldChrMan};
use eldenring::fd4::FD4TaskData;
use eldenring::param::SP_EFFECT_PARAM_ST;
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
    original_data: HashMap<u32, MoreDifficultData>,
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
    fn new(sp_effect_param: &SP_EFFECT_PARAM_ST) -> Self {
        MoreDifficultData {
            max_hp_rate: sp_effect_param.max_hp_rate(),
            physics_attack_rate: sp_effect_param.physics_attack_rate(),
            magic_attack_rate: sp_effect_param.magic_attack_rate(),
            fire_attack_rate: sp_effect_param.fire_attack_rate(),
            thunder_attack_rate: sp_effect_param.thunder_attack_rate(),
            dark_attack_rate: sp_effect_param.dark_attack_rate(),
        }
    }

    fn update_with_raw_original_data(&self, data: &mut SP_EFFECT_PARAM_ST) {
        data.set_max_hp_rate(self.max_hp_rate);
        data.set_physics_attack_rate(self.physics_attack_rate);
        data.set_magic_attack_rate(self.magic_attack_rate);
        data.set_fire_attack_rate(self.fire_attack_rate);
        data.set_thunder_attack_rate(self.thunder_attack_rate);
        data.set_dark_attack_rate(self.dark_attack_rate);
    }

    fn update_with_original_data_for_multiplier(&self, data: &mut SP_EFFECT_PARAM_ST, multiplier: f32) {
        data.set_max_hp_rate(self.max_hp_rate * multiplier);
        data.set_physics_attack_rate(self.physics_attack_rate * multiplier);
        data.set_magic_attack_rate(self.magic_attack_rate * multiplier);
        data.set_fire_attack_rate(self.fire_attack_rate * multiplier);
        data.set_thunder_attack_rate(self.thunder_attack_rate * multiplier);
        data.set_dark_attack_rate(self.dark_attack_rate * multiplier);
    }
}

impl MoreDifficultER {
    fn new() -> Self {
        Self {
            current_difficulty_level: None,
            config: None,
            hooking_error: None,
            original_data: HashMap::new(),
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
        if self.config.is_none() {
            self.init();
        }

        let Some(_) = unsafe { WorldChrMan::instance() }.ok() else {
            return;
        };
        let Some(flag_man) = unsafe { CSEventFlagMan::instance_mut() }.ok() else {
            return;
        };

        if self.original_data.is_empty() {
            self.original_data = initialize_more_difficult_er_data();
        }

        const BASE_SAVED_DIFFICULTY_FLAG_ID: u32 = 1061460000;
        const MAX_DIFFICULTY_LEVEL: u32 = 5;

        let normal_difficulty_level = self.normal_difficulty_level();

        self.handle_more_difficult_er_menu(flag_man, BASE_SAVED_DIFFICULTY_FLAG_ID, MAX_DIFFICULTY_LEVEL, normal_difficulty_level);
        self.handle_more_difficult_er_difficulty(flag_man, BASE_SAVED_DIFFICULTY_FLAG_ID, MAX_DIFFICULTY_LEVEL, normal_difficulty_level);
    }

    fn handle_more_difficult_er_menu(&self, flag_man: &mut CSEventFlagMan, base_saved_difficulty_flag_id: u32, max_difficulty: u32, normal_difficulty_level: u32) {
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
            flag_man.set_flag(base_saved_difficulty_flag_id + normal_difficulty_level, true);
        }
    }

    fn handle_more_difficult_er_difficulty(&mut self, flag_man: &mut CSEventFlagMan, base_saved_difficulty_flag_id: u32, max_difficulty: u32, normal_difficulty_level: u32) {
        let Some(repo) = unsafe { SoloParamRepository::instance_mut() }.ok() else {
            tracing::error!("SoloParamRepository instance not found, cannot handle more difficult ER difficulty");
            return;
        };
        if let Some(difficulty_level_flag) =
            (0..=max_difficulty)
                .find(|level_flag| flag_man.get_flag(*level_flag + base_saved_difficulty_flag_id))
                .filter(|level| level != &self.current_difficulty_level.unwrap_or(u32::MAX)){
            if difficulty_level_flag == normal_difficulty_level {
                tracing::debug!("Applying normal difficulty level for area SP effects");
                area_sp_effect_ids()
                    .for_each(|sp_effect_id| {
                        if let Some(sp_effect_param) = repo.get_mut::<SpEffectParam>(sp_effect_id) &&
                            let Some(original_data) = self.original_data.get(&sp_effect_id) {
                            original_data.update_with_raw_original_data(sp_effect_param);
                        }
                    });
            }
            else {
                let difficulty_power = difficulty_level_flag as i32 - normal_difficulty_level as i32;
                let level_multiplier = self.more_difficult_er_multiplier();
                let multiplier = level_multiplier.powf(difficulty_power as f32);
                tracing::debug!("Applying multiplier {} for level multiplier {} difficulty level {} and difficulty power {}", multiplier, level_multiplier, difficulty_level_flag, difficulty_power);
                area_sp_effect_ids()
                    .for_each(|sp_effect_id| {
                        if let Some(sp_effect_param) = repo.get_mut::<SpEffectParam>(sp_effect_id) &&
                            let Some(original_data) = self.original_data.get(&sp_effect_id) {
                            original_data.update_with_original_data_for_multiplier(sp_effect_param, multiplier);
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

    fn normal_difficulty_level(&self) -> u32 {
        const DEFAULT_NORMAL_DIFFICULTY_LEVEL: u32 = 2;
        match &self.config {
            None => DEFAULT_NORMAL_DIFFICULTY_LEVEL,
            Some(config) => config.normal_difficulty_level.unwrap_or(DEFAULT_NORMAL_DIFFICULTY_LEVEL),
        }
    }
}

fn area_sp_effect_ids() -> impl Iterator<Item = u32> {
    let base_area = (7000..=7280).step_by(10);
    let base_area_npc = 19351..=19370;
    let dlc_area = (20007000..=20007150).step_by(10);
    let dlc_area_npc = (20007200..=20007350).step_by(10);

    base_area.chain(base_area_npc).chain(dlc_area).chain(dlc_area_npc)
}

fn initialize_more_difficult_er_data() -> HashMap<u32, MoreDifficultData> {
    let Some(repo) = unsafe { SoloParamRepository::instance() }.ok() else {
        tracing::error!("SoloParamRepository instance not found, cannot initialize more difficult ER difficulty");
        return HashMap::new();
    };
    area_sp_effect_ids()
        .filter_map(|sp_effect_id|
            repo.get::<SpEffectParam>(sp_effect_id)
                .map(MoreDifficultData::new)
                .map(|more_difficult_data| (sp_effect_id, more_difficult_data))
        )
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
