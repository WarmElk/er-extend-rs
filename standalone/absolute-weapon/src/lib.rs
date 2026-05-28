mod config;
mod upgrade;
mod patch_weapon_reinforcements;

use crate::config::AbsoluteWeaponConfig;
use crate::patch_weapon_reinforcements::patch_weapon_reinforcements;
use crate::upgrade::PlayerGameDataExtender;
use eldenring::cs::{CSEventFlagMan, CSTaskGroupIndex, CSTaskImp, PlayerGameData, PlayerIns, WorldChrMan};
use eldenring::util::system::wait_for_system_init;
use er_extend_rs_discovery::{discover_probable_main_overhaul_mod, ProbableMainOverhaulMod};
use er_extend_rs_esd::initialize_er_extend_rs_esd_from_config;
use er_extend_rs_rva::HookError;
use fromsoftware_shared::{FromStatic, OwnedPtr, Program, SharedTaskImpExt};
use std::cmp::min;
use std::ptr::NonNull;
use std::time::Duration;
use eldenring::fd4::FD4TaskData;
use tracing::level_filters::LevelFilter;

trait FlagManExtender {
    fn compare_and_set_flag(&mut self, flag: u32, expected: bool, set_to: bool) -> bool;
}

impl FlagManExtender for CSEventFlagMan {
    fn compare_and_set_flag(&mut self, flag: u32, expected: bool, set_to: bool) -> bool {
        let matches = self.virtual_memory_flag.get_flag(flag) == expected;
        if matches {
            self.virtual_memory_flag.set_flag(flag, set_to);
        }
        matches
    }
}

#[derive(Default)]
struct WeaponUpgrades {
    highest_regular_weapon_level: u8,
    max_regular_weapon_upgrade_level: u8,
    weapons_upgraded_last_time: i32,
}

#[derive(Default)]
struct Initialization {
    hooking_error: Option<HookError>,
    world_initialized: bool,
    patch_weapon_reinforcements: bool,
    config: Option<AbsoluteWeaponConfig>,
}

#[derive(Default)]
struct Toggles {
    show_debug_window_overlay: bool,
}

struct AbsoluteWeapon {
    initialization: Initialization,
    toggles: Toggles,
    weapon_upgrades: WeaponUpgrades,
}

impl WeaponUpgrades {
    fn update_max_regular_weapon_upgrade_level(&mut self, overhaul: &ProbableMainOverhaulMod) {
        self.max_regular_weapon_upgrade_level = if let ProbableMainOverhaulMod::Reborn = overhaul { 10 } else { 25 };
    }

    fn update_highest_regular_weapon_level_achieved(&mut self, player_game_data: &PlayerGameData) {
        self.highest_regular_weapon_level = min(player_game_data.matching_weapon_level, self.max_regular_weapon_upgrade_level);
    }
}

impl AbsoluteWeapon {
    fn new() -> Self {
        wait_for_system_init(&Program::current(), Duration::MAX).expect("Could not await system init.");
        Self {
            initialization: Initialization::default(),
            toggles: Toggles::default(),
            weapon_upgrades: WeaponUpgrades::default(),
        }
    }

    fn world_initialized(&mut self) -> bool {
        match discover_probable_main_overhaul_mod() {
            Some(overhaul) => {
                if self.initialization.patch_weapon_reinforcements {
                    patch_weapon_reinforcements(&overhaul);
                }
                self.weapon_upgrades.update_max_regular_weapon_upgrade_level(&overhaul);
                tracing::debug!("World initialized with overhaul: {:?}", overhaul);
                true
            }
            None => {
                tracing::debug!("World not initialized");
                false
            }
        }
    }

    fn find_player_game_data(&self) -> Option<NonNull<PlayerGameData>> {
        let player = self.find_player()?;
        Some(player.player_game_data)
    }

    fn find_player(&self) -> Option<&OwnedPtr<PlayerIns>> {
        let world = unsafe { WorldChrMan::instance() }.ok()?;
        let player = world.main_player.as_ref()?;
        Some(player)
    }

    fn reset(&mut self) {
        self.initialization.world_initialized = false;
        self.toggles.show_debug_window_overlay = false;
        self.weapon_upgrades.highest_regular_weapon_level = 0;
        self.weapon_upgrades.weapons_upgraded_last_time = 0;
    }

    fn initialize_esd_config(&mut self) {
        if let Some(ref config) = self.initialization.config &&
            let Err(hook_error) = initialize_er_extend_rs_esd_from_config(&config.extra_config) {

            tracing::error!("Failed to initialize additional grace menu hook: {:?}", hook_error);
            self.initialization.hooking_error = Some(hook_error);
        }
    }
}

impl AbsoluteWeapon {
    fn init(&mut self) {
        let config = config::get_config();

        let logging_level = if config.log_debug_messages.unwrap_or(false) { LevelFilter::DEBUG } else { LevelFilter::WARN };

        tracing_subscriber::fmt::fmt()
            .with_max_level(logging_level)
            .compact()
            .init();

        tracing::debug!("Config: {:?}", config);

        self.initialization.patch_weapon_reinforcements = config.patch_weapon_reinforcements.unwrap_or(true);
        self.initialization.config = Some(config);

        self.initialize_esd_config();
    }

    fn step(&mut self) {
        if self.initialization.config.is_none() {
            self.init();
        }

        let player_game_data = match self.find_player_game_data() {
            Some(mut player_game_data) => unsafe { player_game_data.as_mut() },
            None => {
                self.reset();
                return
            },
        };

        if !self.initialization.world_initialized {
            self.initialization.world_initialized = self.world_initialized();
        }

        self.weapon_upgrades.update_highest_regular_weapon_level_achieved(player_game_data);

        {
            let Some(flag_man) = unsafe { CSEventFlagMan::instance_mut() }.ok() else {
                return;
            };

            if flag_man.compare_and_set_flag(config::UPGRADE_ALL_WEAPONS_FLAG_ID, true, false) {
                self.weapon_upgrades.weapons_upgraded_last_time = player_game_data.upgrade_held_weapons_to_equivalent_level(self.weapon_upgrades.highest_regular_weapon_level);
            }
        }
    }
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

        let mut absolute_weapon = AbsoluteWeapon::new();

        let cs_task = unsafe { CSTaskImp::instance().unwrap() };
        cs_task.run_recurring(
            move |_: &FD4TaskData| {
                absolute_weapon.step();
            },
            CSTaskGroupIndex::FrameBegin,
        );
    });

    true
}
