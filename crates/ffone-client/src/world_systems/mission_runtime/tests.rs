use std::{path::Path, sync::OnceLock};

use crate::assets::AssetLocator;

use crate::world_mission_runtime::*;

mod operations_published_defeat_tasks_are_available_accepta;
mod operations_kill_and_timer_auto_end_edges_register_once;
mod input;
mod output;
mod state;
mod codec;
mod commands;
mod validation;
mod systems;

use operations_published_defeat_tasks_are_available_accepta::{production_content, active_task};
use output::{write_i32_at, write_i64_at, write_running_quest};
