use crate::game_state::Resources;
use serde::{Deserialize, Serialize};

/// Common interface for all purchasable upgrades.
pub trait Upgrade: dyn_clone::DynClone {
    /// Human-readable name of this upgrade.
    fn name(&self) -> &'static str;
    /// The unlock tier this upgrade belongs to.
    fn group(&self) -> u8;
    /// The player's current level for this upgrade (0-based).
    fn get_level(&self) -> u8;
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Human-readable description of the current effect value.
    fn value_text(&self) -> &'static str;
    /// Cost to advance from the current level to the next, or `None` if already maxed.
    fn next_level_cost(&self) -> Option<Resources>;

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
                .take(self.get_level() as usize)
                .collect::<String>(),
            std::iter::repeat(empty_box)
                .take((self.max_level() - self.get_level()) as usize)
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

impl dyn Upgrade + '_ {
    pub fn next_level(&self) -> Option<Box<dyn Upgrade + '_>> {
        if self.get_level() == self.max_level() {
            None
        } else {
            let mut other = dyn_clone::clone_box(self);
            other.level_up();
            Some(other)
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
    pub unlock_brk: UnlockBrk,
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
            &self.bronze_per_instruction,
            &self.code_line_width,
            &self.code_line_count,
            &self.literals,
        ] as [&dyn Upgrade; _]
    }

    fn upgrades_mut(&mut self) -> [&mut dyn Upgrade; 6] {
        [
            &mut self.compile_time,
            &mut self.speed_up_per_instruction_constant,
            &mut self.bronze_per_instruction,
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
            fn fail_oob(&self) -> ! {
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
                Self::value_at(self.0).unwrap_or_else(|| self.fail_oob())
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
                Self::value_text_at(self.0).unwrap_or_else(|| self.fail_oob())
            }

            fn next_level_cost(&self) -> Option<Resources> {
                Self::cost_at(self.0)
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
        (10, Resources::from_bronze(100.), "10"),
        (15, Resources::from_bronze(100e3), "15"),
        (20, Resources::from_bronze(10e6), "20"),
        (30, Resources::zero(), "30"),
    ]
);

impl_upgrade!(
    CodeLineCount,
    type=u8,
    level=1,
    [
        (3, Resources::from_bronze(100.), "3"),
        (5, Resources::from_bronze(100e3), "5"),
        (8, Resources::from_bronze(10e6), "8"),
        (10, Resources::zero(), "10"),
    ]
);

impl_upgrade!(
    LoopStatements,
    type=bool,
    level=1,
    [
        (false, Resources::from_bronze(50e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    MaxInstructions,
    type=u32,
    level=1,
    [
        (100, Resources::from_bronze(50.), "100"),
        (500, Resources::from_bronze(1e3), "500"),
        (2000, Resources::from_bronze(50e3), "2000"),
        (10000, Resources::from_bronze(5e6), "10000"),
        (100000, Resources::zero(), "100000"),
    ]
);

impl_upgrade!(
    CodeExpressionLiterals,
    type=bool,
    level=1,
    [
        (false, Resources::from_bronze(200.), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    UnlockLevel2,
    type=bool,
    level=1,
    [
        (false, Resources::from_bronze(100e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

// Level 2

impl_upgrade!(
    SpeedUpPerInstructionLinear,
    type=u32,
    level=2,
    [
        (1, Resources::from_bronze(50e3), "1"),
        (2, Resources::from_bronze(500e3), "2"),
        (4, Resources::from_silver(5.), "4"),
        (8, Resources::from_silver(50.), "8"),
        (16, Resources::zero(), "16"),
    ]
);

impl_upgrade!(
    BronzePerInstruction,
    type=u32,
    level=2,
    [
        (1, Resources::from_bronze(10.), "1"),
        (5, Resources::from_bronze(100.), "5"),
        (25, Resources::from_bronze(2e3), "25"),
        (125, Resources::from_bronze(30e3), "125"),
        (625, Resources::zero(), "625"),
    ]
);

impl_upgrade!(
    CodeStatements,
    type=bool,
    level=2,
    [
        (false, Resources::from_bronze(500e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    UnlockReboot,
    type=bool,
    level=2,
    [
        (false, Resources::from_silver(100.), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    KeepPrestigeUpgrades,
    type=bool,
    level=2,
    [
        (false, Resources::from_silver(500.), "no"),
        (true, Resources::zero(), "yes"),
    ]
);

impl_upgrade!(
    UnlockLevel3,
    type=bool,
    level=2,
    [
        (false, Resources::from_silver(1e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

// Level 3

impl_upgrade!(
    AutoCompile,
    type=bool,
    level=3,
    [
        (false, Resources::from_silver(5e3), "off"),
        (true, Resources::zero(), "on"),
    ]
);

impl_upgrade!(
    UnlockPrint,
    type=bool,
    level=3,
    [
        (false, Resources::from_silver(10e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    PrintSpeedReset,
    type=f32,
    level=3,
    [
        (1.0, Resources::from_silver(20e3), "1s"),
        (0.5, Resources::from_silver(100e3), "0.5s"),
        (0.1, Resources::zero(), "0.1s"),
    ]
);

impl_upgrade!(
    SilverPerPrintCharacterPolynomial,
    type=f32,
    level=3,
    [
        (1.0, Resources::from_silver(50e3), "1"),
        (2.0, Resources::from_silver(500e3), "2"),
        (5.0, Resources::from_gold(1.), "5"),
        (10.0, Resources::zero(), "10"),
    ]
);

impl_upgrade!(
    UnlockLevel4,
    type=bool,
    level=3,
    [
        (false, Resources::from_gold(10.), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

// Level 4

impl_upgrade!(
    UnlockSleep,
    type=bool,
    level=4,
    [
        (false, Resources::from_gold(50.), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    MinInstructionDuration,
    type=f32,
    level=4,
    [
        (1.0, Resources::from_gold(100.), "1ms"),
        (0.1, Resources::from_gold(1e3), "0.1ms"),
        (0.01, Resources::zero(), "0.01ms"),
    ]
);

impl_upgrade!(
    InstructionSpeedToSleep,
    type=f32,
    level=4,
    [
        (1.0, Resources::from_gold(200.), "1x"),
        (2.0, Resources::from_gold(2e3), "2x"),
        (5.0, Resources::zero(), "5x"),
    ]
);

impl_upgrade!(
    GoldPerSleepSecond,
    type=f32,
    level=4,
    [
        (1.0, Resources::from_gold(500.), "1"),
        (5.0, Resources::from_gold(5e3), "5"),
        (25.0, Resources::zero(), "25"),
    ]
);

impl_upgrade!(
    UnlockLevel5,
    type=bool,
    level=4,
    [
        (false, Resources::from_gold(10e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

// Level 5

impl_upgrade!(
    AutoRun,
    type=bool,
    level=5,
    [
        (false, Resources::from_gold(50e3), "off"),
        (true, Resources::zero(), "on"),
    ]
);

impl_upgrade!(
    UnlockBrk,
    type=bool,
    level=5,
    [
        (false, Resources::from_gold(100e3), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    BreakSlowdown,
    type=f32,
    level=5,
    [
        (2.0, Resources::from_gold(500e3), "2x"),
        (5.0, Resources::from_gold(5e6), "5x"),
        (10.0, Resources::zero(), "10x"),
    ]
);

impl_upgrade!(
    DiamondPerBreakPoint,
    type=f32,
    level=5,
    [
        (1.0, Resources::from_gold(1e6), "1"),
        (5.0, Resources::from_gold(10e6), "5"),
        (25.0, Resources::zero(), "25"),
    ]
);

impl_upgrade!(
    UnlockLevel6,
    type=bool,
    level=5,
    [
        (false, Resources::from_gold(100e6), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

// Level 6

impl_upgrade!(
    GainCurrencyFunction,
    type=bool,
    level=6,
    [
        (false, Resources::from_gold(1e9), "locked"),
        (true, Resources::zero(), "unlocked"),
    ]
);

impl_upgrade!(
    WinCondition,
    type=bool,
    level=6,
    [
        (false, Resources::from_gold(1e12), "not won"),
        (true, Resources::zero(), "won"),
    ]
);
