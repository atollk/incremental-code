use crate::game_state::Resources;
use serde::{Deserialize, Serialize};

/// Common interface for all purchasable upgrades.
pub trait Upgrade {
    /// Human-readable name of this upgrade.
    fn name(&self) -> &'static str;
    /// The unlock tier this upgrade belongs to.
    fn group(&self) -> u8;
    /// The player's current level for this upgrade (0-based).
    fn get_level(&self) -> u8;
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Human-readable description of the current effect value.
    fn value_text(&self) -> String;

    /// Create a new object for the next level instance.
    fn next_level(&self) -> Option<Self> {
        if self.get_level() == self.max_level() {
            None
        } else {
            let mut other = self.clone();
            other.level_up();
            Some(other)
        }
    }

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
    pub code_line_width: CodeLineWidth,
    pub code_line_count: CodeLineCount,
    pub max_instructions: MaxInstructions,
    pub literals: CodeExpressionLiterals,
    pub unlock_level2: UnlockLevel2,
    // Level 2
    pub speed_up_per_instruction_linear: SpeedUpPerInstructionLinear,
    pub bronze_per_instruction: BronzePerInstruction,
    pub statements: CodeStatements,
    pub unlock_reboot: UnlockReboot,
    pub keep_prestige_upgrades: KeepPrestigeUpgrades,
    pub unlock_level3: UnlockLevel3,
    // Level 3
    pub auto_compile: AutoCompile,
    pub unlock_print: UnlockPrint,
    pub print_speed_reset: PrintSpeedReset,
    pub silver_per_print_character_linear: SilverPerPrintCharacterPolynomial,
    pub unlock_level4: UnlockLevel4,
    // Level 4
    pub unlock_sleep: UnlockSleep,
    pub min_instruction_duration: MinInstructionDuration,
    pub instruction_speed_to_sleep: InstructionSpeedToSleep,
    pub gold_per_sleep_second: GoldPerSleepSecond,
    pub unlock_level5: UnlockLevel5,
    // Level 5
    pub auto_run: AutoRun,
    pub unlock_break: UnlockBreak,
    pub break_slowdown: BreakSlowdown,
    pub diamond_per_break: DiamondPerBreakPoint,
    pub unlock_level6: UnlockLevel6,
    // Level 6
    pub gain_currency_function: GainCurrencyFunction,
    pub win_condition: WinCondition,
}

impl Upgrades {
    /// Returns all upgrades as an array of trait-object references.
    pub fn upgrades(&self) -> [&dyn Upgrade; 6] {
        [
            &self.compile_time,
            &self.speed_up_per_instruction_constant,
            &self.bronze_per_instruction_linear,
            &self.code_line_width,
            &self.code_line_count,
            &self.literals,
        ] as [&dyn Upgrade; _]
    }

    fn upgrades_mut(&mut self) -> [&mut dyn Upgrade; 6] {
        [
            &mut self.compile_time,
            &mut self.speed_up_per_instruction_constant,
            &mut self.bronze_per_instruction_linear,
            &mut self.code_line_width,
            &mut self.code_line_count,
            &mut self.literals,
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
        [ $( ($value:expr, $cost:expr, $text:expr) ),+ $(,)? ]
    ) => {
        #[derive(Debug, Default, Clone, PartialEq, std::hash::Hash, serde::Serialize, serde::Deserialize)]
        pub(crate) struct $struct (u8);

        impl $struct {
            fn fail_oob(&self) {
                panic!(
                    concat!(stringify!($struct), ": level {} out of bounds"),
                    self.0
                )
            }

            pub(crate) fn value_at(level: u8) -> Option<$val> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some($value); }
                    __i += 1;
                )+
                None
            }

            pub(crate) fn cost_at(level: u8) -> Option<Resources> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some($cost); }
                    __i += 1;
                )+
                None
            }

            pub(crate) fn value_text_at(level: u8) -> Option<&'static str> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some($text); }
                    __i += 1;
                )+
                None
            }


            pub(crate) fn value(&self) -> $val {
                Self::value_at(self.0).unwrap_or_else(self.fail_oob)
            }
        }

        impl Upgrade for $struct {
            fn name(&self) -> &'static str {
                stringify!($struct)
            }

            fn group(&self) -> u8 {
                $group_level
            }

            fn get_level(&self) -> u8 {
                self.0
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len().saturating_sub(1) as u8
            }

            fn value_text(&self) -> &'static str {
                Self::value_text_at(self.0).unwrap_or_else(self.fail_oob)
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
        (5., Resources::from_bronze(10.), "5s"),
        (4., Resources::from_bronze(1e3), "4s"),
        (3., Resources::from_bronze(1e6), "3s"),
        (2., Resources::from_bronze(1e9), "2s"),
        (1., Resources::from_silver(10.), "1s"),
        (0.1, Resources::zero(), "0.1s"),
    ]
);

impl_upgrade!(
    SpeedUpPerInstructionConstant,
    type=u32,
    level=1,
    [
        (1, Resources::from_bronze(10.), "1"),
        (2, Resources::from_bronze(100.), "1/2"),
        (8, Resources::from_bronze(2e3), "1/8"),
        (64, Resources::from_bronze(30e3), "1/64"),
        (1024, Resources::zero(), "1/1024"),
    ]
);

impl_upgrade!(
    CodeLineWidth,
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
    CodeLineCount,
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
