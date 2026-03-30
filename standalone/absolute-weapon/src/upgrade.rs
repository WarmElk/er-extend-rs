use eldenring::cs::{CSGaitemImp, EquipInventoryDataListEntry, EquipParamWeapon, GaitemCategory, OptionalItemId, PlayerGameData, ReinforceParamWeapon, SoloParamRepository};
use fromsoftware_shared::FromStatic;
use std::cmp::max;
use std::collections::HashMap;

pub trait PlayerGameDataExtender {
    fn upgrade_held_weapons_to_equivalent_level(&mut self, current_highest_regular_level: u8) -> i32;
}

impl PlayerGameDataExtender for PlayerGameData {
    fn upgrade_held_weapons_to_equivalent_level(&mut self, current_highest_regular_level: u8) -> i32 {
        if current_highest_regular_level == 0 {
            tracing::debug!("Player has not upgraded weapons yet. So nothing to do.");
            return 0;
        }
        tracing::debug!("Player current highest regular weapon level: {}", current_highest_regular_level);

        let Ok(repo) = (unsafe { SoloParamRepository::instance() }) else {
            tracing::error!("SoloParamRepository instance not found. Cannot lookup weapons.");
            return 0;
        };

        let mut reinforceable_weapons: Vec<(&mut EquipInventoryDataListEntry, i16)> = self
            .equipment
            .equip_inventory_data
            .items_data
            .items_mut()
            .filter(|item| {
                let Some(category) = &item.gaitem_handle.category().ok() else {
                    return false
                };
                category == &GaitemCategory::Weapon
            })
            .filter_map(|item| {
                let reinforce_type_id = find_reinforce_type_id_for_weapon(item, repo)?;
                Some((item, reinforce_type_id))
            })
            .collect();

        if reinforceable_weapons.is_empty() {
            tracing::debug!("Player has no reinforceable weapons.");
            return 0;
        }

        upgrade_reinforceable_weapons_to_equivalent_level(&mut reinforceable_weapons, current_highest_regular_level, repo)
    }
}

fn find_reinforce_type_id_for_weapon(weapon: &EquipInventoryDataListEntry, repo: &SoloParamRepository) -> Option<i16> {
    let weapon_id = weapon.item_id.param_id();
    let base_weapon_id = weapon_id / 100 * 100;
    repo.get::<EquipParamWeapon>(base_weapon_id)
        .filter(|equip_param_weapon| equip_param_weapon.reinforce_shop_category() > 0)
        .map(|equip_param_weapon| equip_param_weapon.reinforce_type_id())
}

fn upgrade_reinforceable_weapons_to_equivalent_level(reinforceable_weapons: &mut Vec<(&mut EquipInventoryDataListEntry, i16)>, current_highest_regular_level: u8, repo: &SoloParamRepository) -> i32 {
    let Ok(ga_item_accessor) = (unsafe { CSGaitemImp::instance() }) else {
        tracing::error!("Failed to get ga_item_accessor instance.");
        return 0;
    };

    let mut reinforce_type_id_upgrade_level: HashMap<i16, u8> = HashMap::new();

    let mut weapons_upgraded = 0;
    reinforceable_weapons.iter_mut().for_each(|(weapon, reinforce_type_id)| {
        let upgrade_level = reinforce_type_id_upgrade_level.entry(*reinforce_type_id).or_insert_with(|| find_equivalent_weapon_upgrade_level(reinforce_type_id, current_highest_regular_level, repo));
        tracing::debug!("Found upgrade level for reinforce type ID {} and current_max_weapon_level {}: {}", reinforce_type_id, current_highest_regular_level, upgrade_level);
        let weapon_id = weapon.item_id.param_id();
        let current_level = (weapon_id % 100) as u8;
        if *upgrade_level > current_level && let Some(ga_item) = ga_item_accessor.gaitem_ins_by_handle_mut(&weapon.gaitem_handle) {
            let increase = (*upgrade_level - current_level) as u32;
            let upgraded_item_id = OptionalItemId(ga_item.item_id.0 + increase);
            ga_item.item_id = upgraded_item_id;
            weapon.item_id = upgraded_item_id.as_valid().unwrap();
            tracing::debug!("Upgrading weapon ID {} to level {}", weapon_id, upgrade_level);
            weapons_upgraded += 1;
        }
    });
    weapons_upgraded
}

fn find_equivalent_weapon_upgrade_level(reinforce_type_id: &i16, current_highest_regular_level: u8, repo: &SoloParamRepository) -> u8 {
    tracing::debug!("Finding upgrade level for reinforce type ID: {}, current max level: {}", reinforce_type_id, current_highest_regular_level);
    const NO_UPGRADE: u8 = 0;
    for i in NO_UPGRADE..=current_highest_regular_level {
        let param_id = (*reinforce_type_id + i as i16) as u32;
        let max_reinforce_level = repo
            .get::<ReinforceParamWeapon>(param_id)
            .map_or(0, |equip_param_weapon| equip_param_weapon.max_reinforce_level());
        if max_reinforce_level == current_highest_regular_level {
            return i;
        }
        else if max_reinforce_level > current_highest_regular_level {
            return max(i - 1, NO_UPGRADE)
        }
    }
    NO_UPGRADE
}