use eldenring::cs::{SoloParamRepository, TutorialParam, WorldChrMan};
use eldenring::param::TUTORIAL_PARAM_ST;
use fromsoftware_shared::FromStatic;

#[derive(Debug)]
pub enum ProbableMainOverhaulMod {
    Reborn,
    NoKnownOverhaulMod,
}

pub fn discover_probable_main_overhaul_mod() -> Option<ProbableMainOverhaulMod> {
    let _ = unsafe { WorldChrMan::instance() }.ok()?;

    let repo = unsafe { SoloParamRepository::instance() }.ok()?;

    if is_probably_reborn(repo) {
        return Some(ProbableMainOverhaulMod::Reborn);
    }
    Some(ProbableMainOverhaulMod::NoKnownOverhaulMod)
}

fn is_probably_reborn(repo: &SoloParamRepository) -> bool {
    const TUTORIAL_PARAM_IDS_TO_CHECK: &[u32] = &[1010, 1530, 1550];
    const EXPECTED_UNLOCK_EVENT_WITCHING_HOUR_TEXT_ID: i32 = 500000;

    TUTORIAL_PARAM_IDS_TO_CHECK
        .iter()
        .filter_map(|param_id| repo.get::<TutorialParam>(*param_id))
        .map(|tutorial_param: &TUTORIAL_PARAM_ST| tutorial_param.text_id())
        .find(|text_id: &i32| text_id == &EXPECTED_UNLOCK_EVENT_WITCHING_HOUR_TEXT_ID)
        .is_some()
}
