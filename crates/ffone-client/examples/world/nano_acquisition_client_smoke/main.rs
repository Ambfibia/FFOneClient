//! Uses the production client content projection to choose the actual tune
//! request. The wire-only smoke cannot establish client/server XDT agreement.
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::ui_startup;
use ffone_client::{
    assets::AssetLocator,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    nano_free_tuning_runtime::{NanoFreeTuningBank0104, project_nano_free_tuning_content},
    nano_free_tuning_ui::{
        NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS, NanoFreeTuningIntent, NanoFreeTuningModel,
        NanoFreeTuningOpenContext, NanoFreeTuningWorldSnapshot,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use std::{env, error::Error, path::PathBuf};

#[allow(dead_code)]
#[path = "../../../../ffone-net/examples/nano_acquisition_smoke.rs"]
mod wire_smoke;

#[derive(Default)]
struct ClientBankObserver(NanoFreeTuningBank0104);

impl wire_smoke::AcquisitionObserver for ClientBankObserver {
    fn restored(&mut self, nano_id: i16, skill_id: i16) -> Result<(), Box<dyn Error>> {
        assert_eq!(self.0.entries()[nano_id as usize].skill_id, skill_id);
        assert_ne!(self.0.first_untuned_index(), Some(nano_id));
        Ok(())
    }

    fn bootstrap(
        &mut self,
        load: &ffone_protocol::PcLoadData0104,
        player_id: i32,
        bootstrap: &ffone_net::WorldBootstrap,
    ) -> Result<(), Box<dyn Error>> {
        self.0.seed_with_bootstrap(load, player_id, bootstrap)?;
        eprintln!(
            "client world-entry bank: {} records",
            self.0.entries().len()
        );
        Ok(())
    }

    fn created(
        &mut self,
        reply: &ffone_protocol::PcNanoCreateSuccess0104,
    ) -> Result<(), Box<dyn Error>> {
        self.0.apply_create_success(*reply)?;
        assert_eq!(self.0.entries()[reply.nano.id as usize], reply.nano);
        assert!(self.0.first_untuned_index().is_some());
        Ok(())
    }

    fn tuned(&mut self, reply: &ffone_protocol::NanoTuneSuccess0104) -> Result<(), Box<dyn Error>> {
        self.0.apply_tune_success(reply.nano_id, reply.skill_id)?;
        assert_eq!(
            self.0.entries()[reply.nano_id as usize].skill_id,
            reply.skill_id
        );
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let nano_id = env::var("FFONE_TEST_NANO_ID")?.parse::<i16>()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root)?;
    let content = TutorialMissionContent::open(&locator)?;
    assert!(
        content.gameplay_nano(nano_id).is_some(),
        "chat would reject this Nano"
    );
    let catalog = GameplayNanoPortraitCatalog::open(&locator)?;
    let model_path = catalog.model_path(nano_id).ok_or("missing Nano model")?;
    let projected = project_nano_free_tuning_content(&content, nano_id)?;
    eprintln!(
        "Nano {nano_id}: {}; model={model_path}",
        projected.nano_name
    );
    for index in 0..3 {
        let power = &projected.powers[index];
        let mut model = NanoFreeTuningModel::default();
        model.open(NanoFreeTuningOpenContext {
            player_id: 1,
            killed_fusion: false,
            content: projected.clone(),
            world: NanoFreeTuningWorldSnapshot::default(),
        })?;
        model.advance(NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS + 0.01)?;
        model.click_power(index)?;
        let request = model
            .drain_intents()
            .find_map(|intent| match intent {
                NanoFreeTuningIntent::Wire(request) => Some(request),
                _ => None,
            })
            .ok_or("selection did not emit a request")?;
        assert_eq!(request.body.tune_id, power.tune_id);
        eprintln!(
            "card {index}: client sends tune {}, expects skill {}",
            request.body.tune_id, power.skill_id
        );
        wire_smoke::run_with_observer(
            nano_id,
            request.body.tune_id,
            power.skill_id,
            &mut ClientBankObserver::default(),
        )?;
    }
    Ok(())
}
