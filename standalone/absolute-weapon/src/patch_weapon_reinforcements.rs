use eldenring::cs::{ReinforceParamWeapon, SoloParamRepository};
use er_extend_rs_discovery::ProbableMainOverhaulMod;
use fromsoftware_shared::FromStatic;

pub fn patch_weapon_reinforcements(overhaul: &ProbableMainOverhaulMod) {
    match overhaul {
        ProbableMainOverhaulMod::Reborn => {
            patch_weapon_reinforcements_for_reborn();
        }
        ProbableMainOverhaulMod::NoKnownOverhaulMod => {}
    }
}

fn patch_weapon_reinforcements_for_reborn() {
    let Some(repo) = unsafe { SoloParamRepository::instance() }.ok() else {
        return
    };

    const UNIQUE_WEAPON_REINFORCE_TYPE_IDS: &[i16] = &[2200, 2400, 2500, 2600, 2700, 2800, 3200, 3300, 3400, 4000, 4100, 6100, 6200, 6300, 7000, 8300, 8500, 9000];

    for offset in 1u8..=5 {
        let max_reinforce_level = offset * 2;
        UNIQUE_WEAPON_REINFORCE_TYPE_IDS.iter().for_each(|type_id| {
            let type_id_with_offset = (type_id + offset as i16) as u32;
            if let Some(reinforce_param_weapon) = repo.get_mut::<ReinforceParamWeapon>(type_id_with_offset) &&
                reinforce_param_weapon.max_reinforce_level() != max_reinforce_level {

                reinforce_param_weapon.set_max_reinforce_level(max_reinforce_level);
                tracing::debug!("Patched weapon reinforce level for type ID: {}, offset: {}, new level: {}", type_id_with_offset, offset, reinforce_param_weapon.max_reinforce_level());
            }
        });
    }
}