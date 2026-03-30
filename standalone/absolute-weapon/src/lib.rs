mod config;
mod upgrade;
mod patch_weapon_reinforcements;

use crate::config::AbsoluteWeaponConfig;
use crate::patch_weapon_reinforcements::patch_weapon_reinforcements;
use crate::upgrade::PlayerGameDataExtender;
use eldenring::cs::{CSEventFlagMan, PlayerGameData, PlayerIns, WorldChrMan};
use eldenring::util::system::wait_for_system_init;
use er_extend_rs_discovery::{discover_probable_main_overhaul_mod, ProbableMainOverhaulMod};
use er_extend_rs_esd::initialize_er_extend_rs_esd_from_config;
use er_extend_rs_rva::HookError;
use fromsoftware_shared::{FromStatic, OwnedPtr, Program};
use hudhook::hooks::dx12::ImguiDx12Hooks;
use hudhook::{hudhook, ImguiRenderLoop, RenderContext};
use imgui::StyleColor::{Text, WindowBg};
use imgui::{Context, Ui};
use std::cmp::min;
use std::ptr::NonNull;
use std::time::Duration;
use tracing::level_filters::LevelFilter;

trait FlagManExtender {
    fn get_flag(&self, flag_id: u32) -> bool;
    fn compare_and_set_flag(&mut self, flag: u32, expected: bool, set_to: bool) -> bool;
}

impl FlagManExtender for CSEventFlagMan {
    fn get_flag(&self, flag_id: u32) -> bool {
        self.virtual_memory_flag.get_flag(flag_id)
    }

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
    load_menu_on_flag_id: u32,
    menu_loaded: bool,
}

#[derive(Default)]
struct Toggles {
    allow_debug_window_overlay: bool,
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
        self.initialization.menu_loaded = false;
        self.toggles.show_debug_window_overlay = false;
        self.weapon_upgrades.highest_regular_weapon_level = 0;
        self.weapon_upgrades.weapons_upgraded_last_time = 0;
    }

    fn initialize_esd_config(&mut self) {
        if let Some(ref config) = self.initialization.config {
            if let Err(e) = initialize_er_extend_rs_esd_from_config(&config.extra_config) {
                tracing::error!("Failed to initialize additional grace menu hook: {:?}", e);
                self.initialization.hooking_error = Some(e);
            }
            self.initialization.menu_loaded = true;
        }
    }
}

impl ImguiRenderLoop for AbsoluteWeapon {
    fn initialize(&mut self, _ctx: &mut Context, _render_context: &mut dyn RenderContext) {
        let config = config::get_config();

        let logging_level = if config.log_debug_messages.unwrap_or(false) { LevelFilter::DEBUG } else { LevelFilter::WARN };

        tracing_subscriber::fmt::fmt()
            .with_max_level(logging_level)
            .compact()
            .init();

        tracing::debug!("Config: {:?}", config);

        self.toggles.allow_debug_window_overlay = config.allow_debug_window_overlay.unwrap_or(false);
        self.initialization.patch_weapon_reinforcements = config.patch_weapon_reinforcements.unwrap_or(true);

        let mut config = config;
        if !self.toggles.allow_debug_window_overlay {
            config.filter_out_flag_id(config::TOGGLE_UPGRADE_STATS_DISPLAY_FLAG_ID);
        }

        self.initialization.load_menu_on_flag_id = config.load_menu_on_flag_id.unwrap_or_default();

        self.initialization.config = Some(config);
    }

    fn before_render(&mut self, _ctx: &mut Context, _render_context: &mut dyn RenderContext) {
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
            let Some(flag_man) = unsafe { CSEventFlagMan::instance() }.ok() else {
                return;
            };

            if !self.initialization.menu_loaded && (self.initialization.load_menu_on_flag_id == 0 || flag_man.get_flag(self.initialization.load_menu_on_flag_id)) {
                self.initialize_esd_config();
            }

            if !self.initialization.menu_loaded {
                return;
            }

            if self.toggles.allow_debug_window_overlay && flag_man.compare_and_set_flag(config::TOGGLE_UPGRADE_STATS_DISPLAY_FLAG_ID, true, false) {
                self.toggles.show_debug_window_overlay = !self.toggles.show_debug_window_overlay;
            }

            if flag_man.compare_and_set_flag(config::UPGRADE_ALL_WEAPONS_FLAG_ID, true, false) {
                self.weapon_upgrades.weapons_upgraded_last_time = player_game_data.upgrade_held_weapons_to_equivalent_level(self.weapon_upgrades.highest_regular_weapon_level);
            }
        }
    }

    fn render(&mut self, ui: &mut Ui) {
        if !self.initialization.world_initialized || !self.toggles.show_debug_window_overlay {
            return;
        }
        ui.window("##absolute_weapon")
            .no_decoration()
            .no_inputs()
            .no_nav()
            .always_auto_resize(true)
            .build(|| {
                ui.set_window_font_scale(2.0);
                {
                    let background_color = ui.push_style_color(WindowBg, [0.5, 0.5, 0.5, 0.20]);
                    let text_color = ui.push_style_color(Text, [0.0, 1.0, 0.0, 0.50]);
                    ui.text("Absolute Weapon (show/hide at grace)");
                    ui.separator();
                    ui.text(format!("Highest regular weapon level        : {:?}", self.weapon_upgrades.highest_regular_weapon_level));
                    ui.text(format!("Max regular weapon level            : {:?}", self.weapon_upgrades.max_regular_weapon_upgrade_level));
                    ui.text(format!("Number of weapons upgraded last time: {:?}", self.weapon_upgrades.weapons_upgraded_last_time));
                    ui.separator();
                    ui.text(format!("Flag id    : {:?}", self.initialization.load_menu_on_flag_id));
                    ui.text(format!("Menu loaded: {:?}", self.initialization.menu_loaded));
                    if let Some(error) = &self.initialization.hooking_error {
                        ui.separator();
                        ui.text(format!("Hooking error: {:?}", error));
                    }
                    text_color.pop();
                    background_color.pop();
                }
            });
    }
}

hudhook!(ImguiDx12Hooks, AbsoluteWeapon::new());
