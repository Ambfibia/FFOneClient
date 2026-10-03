//! The editor uses the native game's journal, option and NPC portrait assets.
use super::*;
use bevy::sprite::BorderRect;
use ffone_client::gui_skin::gui_style;

pub(super) const TEXT: Color = Color::srgb(0.91, 0.95, 1.0);
pub(super) const MUTED: Color = Color::srgb(0.52, 0.65, 0.79);
pub(super) const CYAN: Color = Color::srgb(0.28, 0.83, 1.0);
pub(super) const BLUE: Color = Color::srgb(0.36, 0.69, 1.0);
pub(super) const DARK: Color = Color::srgb(0.025, 0.041, 0.085);

#[derive(Resource)]
pub(super) struct MissionSkin {
    pub card: Handle<Image>,
    pub frame: Handle<Image>,
    pub speech: Handle<Image>,
    pub message: Handle<Image>,
    pub categories: [Handle<Image>; 3],
    pub portraits: BTreeMap<i64, String>,
    server: AssetServer,
}
impl FromWorld for MissionSkin {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();
        let portraits = world
            .resource::<EditorCatalog>()
            .entries
            .iter()
            .filter(|e| e.kind == CatalogKind::Npc)
            .filter_map(|e| Some((i64::from(e.network_id?), e.icon_path.clone()?)))
            .collect();
        Self {
            card: server.load("ui/en/gameplay/mission/journal/reward_box.png"),
            frame: server.load("ui/en/gameplay/mission/journal/npc_icon_back.png"),
            speech: server.load("ui/en/gameplay/speech/quick-chat-box.png"),
            message: server.load("ui/en/gameplay/nanocom/messagearea.png"),
            categories: [
                server.load("ui/en/gameplay/journal/nrcomicon.png"),
                server.load("ui/en/gameplay/journal/nrnanoicon.png"),
                server.load("ui/en/gameplay/journal/nrworldicon.png"),
            ],
            portraits,
            server: server.clone(),
        }
    }
}
pub(super) fn skin_image(image: Handle<Image>, skin: &str, style: &str) -> ImageNode {
    let border = gui_style(skin, style)
        .map(|s| BorderRect {
            min_inset: Vec2::new(s.border.left as f32, s.border.top as f32),
            max_inset: Vec2::new(s.border.right as f32, s.border.bottom as f32),
        })
        .unwrap_or(BorderRect::all(2.));
    sliced_image(image, border)
}
impl MissionSkin {
    pub fn card(&self) -> ImageNode {
        skin_image(self.card.clone(), "FusionFallMissionSkin", "textField")
    }
    pub fn category(&self, value: &Value) -> Handle<Image> {
        self.categories[match value["m_iHMissionType"].as_i64() {
            Some(1) => 0,
            Some(2) => 1,
            _ => 2,
        }]
        .clone()
    }
    pub fn portrait(&self, value: &Value) -> Option<Handle<Image>> {
        ["m_iHJournalNPCID", "m_iHNPCID", "m_iHTerminatorNPCID"]
            .into_iter()
            .filter_map(|field| value[field].as_i64())
            .find_map(|id| {
                self.portraits
                    .get(&id)
                    .map(|path| self.server.load(path.clone()))
            })
    }
    pub fn portrait_badge(&self, p: &mut ChildSpawnerCommands, value: &Value, size: f32) {
        self.badge(p, self.portrait(value).unwrap_or_else(|| self.category(value)), size);
    }
    pub fn speaker_badge(&self, p: &mut ChildSpawnerCommands, npc: i64, value: &Value, size: f32) {
        let portrait=self.portraits.get(&npc).map(|path|self.server.load(path.clone()))
            .unwrap_or_else(||self.category(value));
        self.badge(p,portrait,size);
    }
    fn badge(&self, p: &mut ChildSpawnerCommands, portrait: Handle<Image>, size: f32) {
        p.spawn((
            Node {
                width: px(size),
                height: px(size),
                flex_shrink: 0.,
                padding: UiRect::all(px(3)),
                ..default()
            },
            skin_image(self.frame.clone(), "FusionFallMissionSkin", "npcicon"),
            BackgroundColor(Color::srgb(0.02, 0.07, 0.12)),
        ))
        .with_children(|p| {
            p.spawn((
                ImageNode::new(portrait),
                Node {
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
            ));
        });
    }
}
pub(super) fn heading(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    text: String,
    size: f32,
    color: Color,
) {
    let mut bundle = editor_text(f, "ui.editor.xdt.value", "{value}", size, color, true);
    bundle.localized = bundle.localized.with_arg("value", text);
    bundle.layout = TextLayout::new(Justify::Left, LineBreak::NoWrap);
    p.spawn((
        bundle,
        Node {
            min_width: px(0),
            overflow: Overflow::clip(),
            ..default()
        },
    ));
}
