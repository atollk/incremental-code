use crate::game_state::Resources;
use serde::{Deserialize, Serialize};

/// Common interface for all purchasable upgrades.
pub trait Upgrade {
    /// Human-readable name of this upgrade.
    fn name(&self) -> &'static str;
    /// The unlock tier this upgrade belongs to.
    fn group(&self) -> u8;
    /// The player's current level for this upgrade (0-based).
    fn current_level(&self) -> u8;
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Cost to advance from the current level to the next, or `None` if already maxed.
    fn next_level_cost(&self) -> Option<Resources>;
    /// Human-readable description of the current effect value.
    fn current_value_text(&self) -> String;
    /// Human-readable description of the effect value at the next level, or `None` if maxed.
    fn next_value_text(&self) -> Option<String>;

    /// Advance this upgrade by one level, capping at [`max_level`](Self::max_level).
    fn level_up(&mut self);
    /// Reduce this upgrade by one level, clamping at zero.
    fn level_down(&mut self);

    /// Renders the upgrade level as a string of box characters.
    ///
    /// `full_box` is used for purchased levels; `empty_box` for remaining slots.
    fn format_level_str(&self, empty_box: char, full_box: char) -> String {
        format!(
            "{}{}",
            std::iter::repeat(full_box)
                .take(self.current_level() as usize)
                .collect::<String>(),
            std::iter::repeat(empty_box)
                .take((self.max_level() - self.current_level()) as usize)
                .collect::<String>(),
        )
    }

    /// Returns a display string for the next-level cost, or `"maxed"` if at max level.
    fn format_cost_str(&self) -> String {
        match self.next_level_cost() {
            Some(r) => r.fmt_oneline().to_string(),
            None => "maxed".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
/// Container for all in-game upgrades, serialized as part of [`GameState`](crate::game_state::GameState).
pub struct Upgrades {
    // Level 1
    pub compile_time: CompileTime,
    pub speed_up_per_instruction_constant: SpeedUpPerInstructionConstant,
    pub bronze_per_instruction_linear: BronzePerInstructionLinear,
    pub code_line_width_1: CodeLineWidth1,
    pub code_line_count_1: CodeLineCount1,
    pub loop_statements: LoopStatements,
}

impl Upgrades {
    /// Returns all upgrades as an array of trait-object references.
    pub fn upgrades(&self) -> [&dyn Upgrade; 6] {
        [
            &self.compile_time,
            &self.speed_up_per_instruction_constant,
            &self.bronze_per_instruction_linear,
            &self.code_line_width_1,
            &self.code_line_count_1,
            &self.loop_statements,
        ] as [&dyn Upgrade; _]
    }

    fn upgrades_mut(&mut self) -> [&mut dyn Upgrade; 6] {
        [
            &mut self.compile_time,
            &mut self.speed_up_per_instruction_constant,
            &mut self.bronze_per_instruction_linear,
            &mut self.code_line_width_1,
            &mut self.code_line_count_1,
            &mut self.loop_statements,
        ] as [&mut dyn Upgrade; _]
    }

    /// Returns a mutable reference to the upgrade at the given index.
    pub fn upgrade_at_mut(&mut self, index: usize) -> &mut dyn Upgrade {
        self.upgrades_mut()[index]
    }
}

macro_rules! impl_upgrade {
    (
        $struct:ident,
        type=$val:ty,
        level=$group_level:expr,
        [ $( ($value:expr, $cost:expr) ),+ $(,)? ]
    ) => {
        #[derive(Debug, Default, Clone, PartialEq, std::hash::Hash, serde::Serialize, serde::Deserialize)]
        pub(crate) struct $struct (u8);

        impl $struct {
            pub(crate) fn value(level: u8) -> Option<$val> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some($value); }
                    __i += 1;
                )+
                None
            }

            pub(crate) fn current_value(&self) -> $val {
                if let Some(v) = Self::value(self.0) {
                    v
                } else {
                    panic!(
                        concat!(stringify!($struct), ": level {} out of bounds"),
                        self.0
                    )
                }
            }
        }

        impl Upgrade for $struct {
            fn name(&self) -> &'static str {
                stringify!($struct)
            }

            fn group(&self) -> u8 {
                $group_level
            }

            fn current_level(&self) -> u8 {
                self.0
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len().saturating_sub(1) as u8
            }

            #[allow(unused_assignments)]
            fn current_value_text(&self) -> String {
                format!("{}", self.current_value())
            }

            #[allow(unused_assignments)]
            fn next_value_text(&self) -> Option<String> {
                Self::value(self.0.checked_add(1)?).map(|v| format!("{}", v))
            }

            #[allow(unused_assignments)]
            fn next_level_cost(&self) -> Option<Resources> {
                let __next = self.0.checked_add(1)?;
                if __next >= [ $( impl_upgrade!(@unit $value) ),+ ].len() as u8 {
                    return None;
                }
                let mut __i: u8 = 0;
                $(
                    if self.0 == __i { return Some($cost); }
                    __i += 1;
                )+
                None
            }

            fn level_up(&mut self) {
                self.0 = std::cmp::min(self.0 + 1, self.max_level());
            }

            fn level_down(&mut self) {
                self.0 = self.0.saturating_sub(1);
            }
        }
    };
    (@unit $_:expr) => { () };
}

// Level 1

impl_upgrade!(
    CompileTime,
    type=f32,
    level=1,
    [
        (5., Resources::from_bronze(10.)),
        (4., Resources::from_bronze(1e3)),
        (3., Resources::from_bronze(1e6)),
        (2., Resources::from_bronze(1e9)),
        (1., Resources::from_silver(10.)),
        (0.1, Resources::zero()),
    ]
);

impl_upgrade!(
    SpeedUpPerInstructionConstant,
    type=u32,
    level=1,
    [
        (1, Resources::from_bronze(10.)),
        (2, Resources::from_bronze(100.)),
        (8, Resources::from_bronze(2e3)),
        (64, Resources::from_bronze(30e3)),
        (1024, Resources::zero()),
    ]
);

impl_upgrade!(
    BronzePerInstructionLinear,
    type=u32,
    level=1,
    [
        (1, Resources::from_bronze(10.)),
        (2, Resources::from_bronze(100.)),
        (3, Resources::from_bronze(2e3)),
        (4, Resources::from_bronze(30e3)),
        (5, Resources::zero()),
    ]
);
impl_upgrade!(
    CodeLineWidth1,
    type=u8,
    level=1,
    [
        (10, Resources::from_bronze(100.)),
        (15, Resources::from_bronze(100e3)),
        (20, Resources::from_bronze(10e6)),
        (30, Resources::zero()),
    ]
);

impl_upgrade!(
    CodeLineCount1,
    type=u8,
    level=1,
    [
        (3, Resources::from_bronze(100.)),
        (5, Resources::from_bronze(100e3)),
        (8, Resources::from_bronze(10e6)),
        (10, Resources::zero()),
    ]
);

impl_upgrade!(
    LoopStatements,
    type=bool,
    level=1,
    [
        (false, Resources::from_bronze(50e3)),
        (true, Resources::default()),
    ]
);
