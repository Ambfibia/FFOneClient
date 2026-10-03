use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionTextureRole {
    Backdrop,
    Frame,
    Graphics,
    GraphicsHover,
    GraphicsSelected,
    GameUi,
    GameUiHover,
    GameUiSelected,
    Social,
    SocialHover,
    SocialSelected,
    Controls,
    ControlsHover,
    ControlsSelected,
    Close,
    CloseHover,
    Button,
    ButtonHover,
    OuterFrame,
    RadioEmpty,
    RadioChecked,
    RowSelected,
    PanelSmall,
    PanelSide,
    PanelConnector,
    PanelFlat,
    PanelNotch,
    Divider,
    TextField,
    ScrollBar,
    ScrollDown,
    ScrollThumb,
    ScrollUp,
    Pulldown,
    PulldownHover,
    DropdownPanel,
    DropdownItemHover,
    SliderTrack,
    SliderThumb,
    ColorBox,
    ColorSelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionTextureContract {
    pub role: OptionTextureRole,
    pub runtime_path: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
}

pub const OPTION_TEXTURE_CONTRACTS: [OptionTextureContract; 41] = [
    OptionTextureContract {
        role: OptionTextureRole::Backdrop,
        runtime_path: "ui/en/option/backdrop.png",
        bytes: 1_249_122,
        sha256: "fb9c6c8a4b8313766364ff3398070b35d8a4137f511accc4ddaaea91bd0162f1",
    },
    OptionTextureContract {
        role: OptionTextureRole::Frame,
        runtime_path: "ui/en/option/frame.png",
        bytes: 145,
        sha256: "a719da6205a939d444e82d9c813e395871d6e1e0fe2535852918b728dd1aa133",
    },
    OptionTextureContract {
        role: OptionTextureRole::Graphics,
        runtime_path: "ui/en/option/tab-graphics.png",
        bytes: 1_726,
        sha256: "62696664fc6d05f9ee82347a19a5cd5c5ca9c0eac5646f006fa468ebc22a4df1",
    },
    OptionTextureContract {
        role: OptionTextureRole::GraphicsHover,
        runtime_path: "ui/en/option/tab-graphics-hover.png",
        bytes: 1_531,
        sha256: "03fe5097f70691d06b9d69097e6eef701ebac2af2d1a47c26ade115d5a6d9811",
    },
    OptionTextureContract {
        role: OptionTextureRole::GraphicsSelected,
        runtime_path: "ui/en/option/tab-graphics-selected.png",
        bytes: 1_499,
        sha256: "9e374cfcbadea92dae53eb3df9a36153f52c096bf6fa88b8bed3eb874e9e0973",
    },
    OptionTextureContract {
        role: OptionTextureRole::GameUi,
        runtime_path: "ui/en/option/tab-game-ui.png",
        bytes: 1_606,
        sha256: "ecc8bb2f89e398712907572449a9c9221b56f77d635673c6973a0d2d19458eb1",
    },
    OptionTextureContract {
        role: OptionTextureRole::GameUiHover,
        runtime_path: "ui/en/option/tab-game-ui-hover.png",
        bytes: 1_539,
        sha256: "86c1dc50fcf2127d4ba3e5ddcecad4ab6e4910ebfd919d6c5b283402f368bfef",
    },
    OptionTextureContract {
        role: OptionTextureRole::GameUiSelected,
        runtime_path: "ui/en/option/tab-game-ui-selected.png",
        bytes: 1_878,
        sha256: "1df27e302621723a673e112239f547cea4301d2e7b30f01b841c1c4aeb49b66b",
    },
    OptionTextureContract {
        role: OptionTextureRole::Social,
        runtime_path: "ui/en/option/tab-social.png",
        bytes: 1_586,
        sha256: "177d2407da8a08df7cec79ccad2eb17873c801f38bf77d62708685f2ac753d13",
    },
    OptionTextureContract {
        role: OptionTextureRole::SocialHover,
        runtime_path: "ui/en/option/tab-social-hover.png",
        bytes: 1_520,
        sha256: "56d782639ad40ffedede256d209a32cdbc9441485d9f469ff545cd94f6cf3542",
    },
    OptionTextureContract {
        role: OptionTextureRole::SocialSelected,
        runtime_path: "ui/en/option/tab-social-selected.png",
        bytes: 1_948,
        sha256: "b68dfeaecb4e59a4cab672c77cfd3d186be32f6e279cec93c37be50cfb27543f",
    },
    OptionTextureContract {
        role: OptionTextureRole::Controls,
        runtime_path: "ui/en/option/tab-controls.png",
        bytes: 1_374,
        sha256: "ec00c3a83764eddbf7bb3eae81ad6def78f47576923a8ebb3a7ae42772dfafaa",
    },
    OptionTextureContract {
        role: OptionTextureRole::ControlsHover,
        runtime_path: "ui/en/option/tab-controls-hover.png",
        bytes: 1_356,
        sha256: "b1d4b82cc90c3281d4b7384c424bfd24200cb4c7d206ba92311c1bde8299fd73",
    },
    OptionTextureContract {
        role: OptionTextureRole::ControlsSelected,
        runtime_path: "ui/en/option/tab-controls-selected.png",
        bytes: 1_414,
        sha256: "21e76a7569bba527c4a7168f8dc41b141ed4e17da2d50062b87fce0b1f84b870",
    },
    OptionTextureContract {
        role: OptionTextureRole::Close,
        runtime_path: "ui/en/option/close.png",
        bytes: 2_055,
        sha256: "5f5f6bfce0bf796d802714ea75ff49bebf21bb536674143bbcb925a0bd57929d",
    },
    OptionTextureContract {
        role: OptionTextureRole::CloseHover,
        runtime_path: "ui/en/option/close-hover.png",
        bytes: 1_588,
        sha256: "cbbc0520c3ada6ab47311caacf342fa7fb7f0f062279f8793bf2a392a89bee1d",
    },
    OptionTextureContract {
        role: OptionTextureRole::Button,
        runtime_path: "ui/en/option/button.png",
        bytes: 576,
        sha256: "506ea52cf108eeb14a035fa52a4c650ba1a5255b6e4fb62a10adc3aedfc72c8e",
    },
    OptionTextureContract {
        role: OptionTextureRole::ButtonHover,
        runtime_path: "ui/en/option/button-hover.png",
        bytes: 832,
        sha256: "1eff6610db23cffdb5e454708141b56a10a4e60cdfb7d64357efe0c6c30b1798",
    },
    OptionTextureContract {
        role: OptionTextureRole::OuterFrame,
        runtime_path: "ui/en/option/outer-frame.png",
        bytes: 795,
        sha256: "aa28713a861cf2d88a4be26e46fa839f0a381bfc23e8501f453eae5da765cc60",
    },
    OptionTextureContract {
        role: OptionTextureRole::RadioEmpty,
        runtime_path: "ui/en/option/radio-empty.png",
        bytes: 557,
        sha256: "f059fa6f9f24ea54da58d6345ea411108b8e1635b8c104e1ac26cf25fef4b0bc",
    },
    OptionTextureContract {
        role: OptionTextureRole::RadioChecked,
        runtime_path: "ui/en/option/radio-checked.png",
        bytes: 1_021,
        sha256: "4d16ff48e39503081716cb49605d609365f934ee21adac69774a4a4ba2b972ce",
    },
    OptionTextureContract {
        role: OptionTextureRole::RowSelected,
        runtime_path: "ui/en/option/row-selected.png",
        bytes: 110,
        sha256: "6cedbfef16fee0aee5720a4cd0281964e59a53a8c2e67bba3927fb7dd6eb0666",
    },
    OptionTextureContract {
        role: OptionTextureRole::PanelSmall,
        runtime_path: "ui/en/option/panel-small.png",
        bytes: 621,
        sha256: "2dd3fa41f4afbb500429c8822e5cb65ec914da852d236ba40cdc787fc52210b1",
    },
    OptionTextureContract {
        role: OptionTextureRole::PanelSide,
        runtime_path: "ui/en/option/panel-side.png",
        bytes: 522,
        sha256: "8ef65ec45c29b14c1074b59c808aed1b49c886a0e2da970bc37b9b15ab7c1190",
    },
    OptionTextureContract {
        role: OptionTextureRole::PanelConnector,
        runtime_path: "ui/en/option/panel-connector.png",
        bytes: 524,
        sha256: "21e5bfc5d8c0918c407c612f0ea081c5d716389a82febfc9dd636a8ccffc9b18",
    },
    OptionTextureContract {
        role: OptionTextureRole::PanelFlat,
        runtime_path: "ui/en/option/panel-flat.png",
        bytes: 212,
        sha256: "3ebb66b2a6523f234da6ebcb8bdcaf83a2048242b268098f4878e9b5edcc1f16",
    },
    OptionTextureContract {
        role: OptionTextureRole::PanelNotch,
        runtime_path: "ui/en/option/panel-notch.png",
        bytes: 341,
        sha256: "41c7445a3a0c3ff3dd07a00d71e30d9a6f297a367427c9879872dd9bcaf4919d",
    },
    OptionTextureContract {
        role: OptionTextureRole::Divider,
        runtime_path: "ui/en/option/divider.png",
        bytes: 145,
        sha256: "a719da6205a939d444e82d9c813e395871d6e1e0fe2535852918b728dd1aa133",
    },
    OptionTextureContract {
        role: OptionTextureRole::TextField,
        runtime_path: "ui/en/option/textfield.png",
        bytes: 217,
        sha256: "f5a79b4fda32a41147172a2ef4cfccbf62beed87b01f7a31dc4acf9e82cd1556",
    },
    OptionTextureContract {
        role: OptionTextureRole::ScrollBar,
        runtime_path: "ui/en/option/scroll-bar.png",
        bytes: 225,
        sha256: "94cc317466de205d6336038b99626a3596ea5592298ecdc409f4e93c7fb83355",
    },
    OptionTextureContract {
        role: OptionTextureRole::ScrollDown,
        runtime_path: "ui/en/option/scroll-down.png",
        bytes: 481,
        sha256: "de13be5d83a1e28ebd312c1c38f293dc28126514f90e92124084e431b5a149fd",
    },
    OptionTextureContract {
        role: OptionTextureRole::ScrollThumb,
        runtime_path: "ui/en/option/scroll-thumb.png",
        bytes: 318,
        sha256: "a7afff2d82ddc872cc99bc2695039530ce1e1a554dceb95f33e344804ef589ad",
    },
    OptionTextureContract {
        role: OptionTextureRole::ScrollUp,
        runtime_path: "ui/en/option/scroll-up.png",
        bytes: 525,
        sha256: "4b1a86ad7cd3d5a885f1e0137769d0de9f3ca2226572573bf541e14278f660ee",
    },
    OptionTextureContract {
        role: OptionTextureRole::Pulldown,
        runtime_path: "ui/en/option/pulldown.png",
        bytes: 887,
        sha256: "a4f08a503436d5f4982f7769626a250fc4614484d3a21b5f095397d2d95abcf1",
    },
    OptionTextureContract {
        role: OptionTextureRole::PulldownHover,
        runtime_path: "ui/en/option/pulldown-hover.png",
        bytes: 981,
        sha256: "f6f468d73564f7bd574f74e040f98c81c179337dc1818613c38f95d499d5a34e",
    },
    OptionTextureContract {
        role: OptionTextureRole::DropdownPanel,
        runtime_path: "ui/en/option/dropdown-panel.png",
        bytes: 1_721,
        sha256: "9f2d695bf6efc5e82fa4bff491c1ad2a6b99ed0350be50f0d41c5e74d9df7041",
    },
    OptionTextureContract {
        role: OptionTextureRole::DropdownItemHover,
        runtime_path: "ui/en/option/dropdown-item-hover.png",
        bytes: 179,
        sha256: "6b02648f8739d26e520ea46d36f847e2cd9d9240891da4a9b561cd0c7eb5717c",
    },
    OptionTextureContract {
        role: OptionTextureRole::SliderTrack,
        runtime_path: "ui/en/option/slider-track.png",
        bytes: 2_664,
        sha256: "8143920345483e4f76ec148e8dd5e412b07238e2da7c7c96639d8105e50b5385",
    },
    OptionTextureContract {
        role: OptionTextureRole::SliderThumb,
        runtime_path: "ui/en/option/slider-thumb.png",
        bytes: 515,
        sha256: "d766bf4746f19d0bb711a93ca990cffb389578f2b1b02794254d5069eff4d849",
    },
    OptionTextureContract {
        role: OptionTextureRole::ColorBox,
        runtime_path: "ui/en/option/color-box.png",
        bytes: 247,
        sha256: "379e6653299419dbc90d44f222b83b478d3e6568c1d5704590de24b3ad24569d",
    },
    OptionTextureContract {
        role: OptionTextureRole::ColorSelected,
        runtime_path: "ui/en/option/color-selected.png",
        bytes: 816,
        sha256: "f9aaab8f262f615db64181a9283dcebbb24ec796527c432f5e95598d917acaa7",
    },
];

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum TextureQuality {
    #[default]
    High,
    Medium,
    Low,
}

impl TextureQuality {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}
