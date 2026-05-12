use crate::game_state::Resources;
use helper_macros::FieldsAs;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Common interface for all purchasable upgrades.
pub trait Upgrade: dyn_clone::DynClone {
    /// Human-readable name of this upgrade.
    fn name(&self) -> &'static str;
    /// The unlock tier this upgrade belongs to.
    fn group(&self) -> usize;
    /// The player's current level for this upgrade (0-based).
    fn get_level(&self) -> u8;
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Human-readable description of the current effect value.
    fn value_text(&self) -> Cow<'static, str>;
    /// Cost to advance from the current level to the next, or `None` if already maxed.
    fn next_level_cost(&self) -> Option<Resources>;

    /// Advance this upgrade by one level, capping at [`max_level`](Self::max_level).
    fn level_up(&mut self);
    /// Reduce this upgrade by one level, clamping at zero.
    fn level_down(&mut self);

    /// Renders the upgrade level as a string of box characters.
    fn format_level_str(&self) -> String {
        match self.max_level() {
            0 => unreachable!(),
            1 => (if self.get_level() == 0 { "[ ]" } else { "[x]" }).to_string(),
            n => format!("{} / {}", self.get_level(), n),
        }
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

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, FieldsAs)]
#[fields_as(Upgrade)]
/// Container for all in-game upgrades, serialized as part of [`GameState`](crate::game_state::GameState).
pub struct Upgrades {
    // Level 0
    pub unlock_code: UnlockCode,
    pub unlock_hud: UnlockHud,
    pub unlock_music: UnlockMusic,
    pub unlock_level1: UnlockLevel1,
    // Level 1
    pub compile_time: CompileTime,
    pub instruction_execution_speed: InstructionExecutionSpeed,
    pub code_line_width: CodeLineWidth,
    pub code_line_count: CodeLineCount,
    pub max_instructions: MaxInstructions,
    pub literals: CodeExpressionLiterals,
    pub unlock_level2: UnlockLevel2,
    // Level 2
    pub bronze_per_instruction: BronzePerInstruction,
    pub statements: CodeStatements,
    pub unlock_reboot: UnlockReboot,
    pub keep_prestige_upgrades: KeepPrestigeUpgrades,
    pub unlock_level3: UnlockLevel3,
    // Level 3
    pub auto_compile: AutoCompile,
    pub unlock_print: UnlockPrint,
    pub print_speed_reset: PrintSpeedReset,
    pub silver_per_print_character: SilverPerPrintCharacter,
    pub resources_after_reboot: RessourcesAfterReboot,
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
    pub fn upgrades(&self) -> [&dyn Upgrade; 34] {
        self.fields_as()
    }

    /// Returns a mutable reference to the upgrade at the given index.
    pub fn upgrade_at_mut(&mut self, index: usize) -> &mut dyn Upgrade {
        self.fields_as_mut()[index]
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

            pub(crate) fn value_text_at(level: u8) -> Option<Cow<'static, str>> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some(Cow::from($text)); }
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

            fn group(&self) -> usize {
                $group_level
            }

            fn get_level(&self) -> u8 {
                self.0
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len().saturating_sub(1) as u8
            }

            fn value_text(&self) -> Cow<'static, str> {
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

const LOCKED: &str = "🔒";
const UNLOCKED: &str = "🔓";

/*
 */

// Level 0

impl_upgrade!(
    UnlockCode,
    type=bool,
    level=0,
    [
        (false, Resources::from_bronze(0.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    UnlockHud,
    type=bool,
    level=0,
    [
        (false, Resources::from_bronze(1.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    UnlockMusic,
    type=bool,
    level=0,
    [
        (false, Resources::from_bronze(5.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    UnlockLevel1,
    type=bool,
    level=0,
    [
        (false, Resources::from_bronze(5.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 1

impl_upgrade!(
    CompileTime,
    type=f32,
    level=1,
    [
        (10., Resources::from_bronze(10.), "10s"),
        (5., Resources::from_bronze(100.), "5s"),
        (4., Resources::from_bronze(1e3), "4s"),
        (3., Resources::from_bronze(1e6), "3s"),
        (2., Resources::from_bronze(1e9), "2s"),
        (1., Resources::from_silver(10.), "1s"),
        (0.1, Resources::zero(), "0.1s"),
    ]
);

impl_upgrade!(
    InstructionExecutionSpeed,
    type=u32,
    level=1,
    [
        (1, Resources::from_bronze(50.), "100 %"),
        (1, Resources::from_bronze(50.), "90 %"),
        (1, Resources::from_bronze(50.), "80 %"),
        (1, Resources::from_bronze(50.), "70 %"),
        (1, Resources::from_bronze(50.), "60 %"),
        (1, Resources::from_bronze(50.), "50 %"),
        (2, Resources::from_bronze(100.), "40 %"),
        (2, Resources::from_bronze(100.), "30 %"),
        (4, Resources::from_bronze(100.), "25 %"),
        (4, Resources::from_bronze(100.), "20 %"),
        (4, Resources::from_bronze(100.), "15 %"),
        (10, Resources::from_bronze(30e3), "10 %"),
        (10, Resources::from_bronze(30e3), "5 %"),
        (10, Resources::from_bronze(30e3), "2.5 %"),
        (10, Resources::from_bronze(30e3), "1 %"),
        (10, Resources::from_bronze(30e3), "0.5 %"),
        (10, Resources::from_bronze(30e3), "0.25 %"),
        (10, Resources::from_bronze(30e3), "0.1 %"),
        (10, Resources::from_bronze(30e3), "n ^ -0.1"),
        (10, Resources::from_bronze(30e3), "n ^ -0.2"),
        (10, Resources::from_bronze(30e3), "n ^ -0.3"),
        (10, Resources::from_bronze(30e3), "n ^ -0.4"),
        (10, Resources::from_bronze(30e3), "n ^ -0.5"),
        (10, Resources::from_bronze(30e3), "n ^ -0.6"),
        (10, Resources::from_bronze(30e3), "n ^ -0.7"),
        (10, Resources::from_bronze(30e3), "n ^ -0.8"),
        (10, Resources::from_bronze(30e3), "n ^ -0.9"),
        (10, Resources::from_bronze(30e3), "n ^ -1"),
    ]
);

impl_upgrade!(
    CodeLineWidth,
    type=u8,
    level=1,
    [
        (5, Resources::from_bronze(5.), "5"),
        (10, Resources::from_bronze(100.), "10"),
        (15, Resources::from_bronze(100e3), "15"),
        (30, Resources::from_bronze(10e6), "30"),
        (50, Resources::from_silver(100.), "50"),
        (80, Resources::zero(), "80"),
    ]
);

impl_upgrade!(
    CodeLineCount,
    type=u8,
    level=1,
    [
        (1, Resources::from_bronze(5.), "1"),
        (2, Resources::from_bronze(25.), "2"),
        (4, Resources::from_bronze(1e3), "4"),
        (4, Resources::from_bronze(1e3), "5"),
        (6, Resources::from_bronze(100e3), "6"),
        (6, Resources::from_bronze(100e3), "7"),
        (8, Resources::from_bronze(10e6), "8"),
        (10, Resources::from_silver(10.), "10"),
        (10, Resources::from_silver(10.), "15"),
        (20, Resources::from_silver(1e3), "20"),
        (30, Resources::from_gold(100.), "30"),
        (40, Resources::zero(), "40"),
    ]
);

impl_upgrade!(
    CodeStatements,
    type=(),
    level=1,
    [
        ((), Resources::from_bronze(500e3), ""),
        ((), Resources::from_bronze(500e3), "simple loops"),
        ((), Resources::from_bronze(500e3), "nested loops"),
        ((), Resources::from_bronze(500e3), "functions"),
        ((), Resources::from_bronze(500e3), "single recursion"),
        ((), Resources::zero(), "multi recursion"),
    ]
);

impl_upgrade!(
    UnlockLevel2,
    type=bool,
    level=1,
    [
        (false, Resources::from_bronze(100e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 2

impl_upgrade!(
    BronzePerInstruction,
    type=u32,
    level=2,
    [
        (1, Resources::from_bronze(10.), "1"),
        (1, Resources::from_bronze(10.), "2"),
        (1, Resources::from_bronze(10.), "3"),
        (1, Resources::from_bronze(10.), "4"),
        (5, Resources::from_bronze(100.), "5"),
        (5, Resources::from_bronze(100.), "6"),
        (5, Resources::from_bronze(100.), "7"),
        (5, Resources::from_bronze(100.), "8"),
        (5, Resources::from_bronze(100.), "9"),
        (5, Resources::from_bronze(100.), "10"),
        (5, Resources::from_bronze(100.), "15"),
        (5, Resources::from_bronze(100.), "20"),
        (5, Resources::from_bronze(100.), "25"),
        (5, Resources::from_bronze(100.), "30"),
        (5, Resources::from_bronze(100.), "35"),
        (5, Resources::from_bronze(100.), "40"),
        (5, Resources::from_bronze(100.), "50"),
        (5, Resources::from_bronze(100.), "100"),
        (5, Resources::from_bronze(100.), "n"),
        (5, Resources::from_bronze(100.), "n ^ 1.5"),
        (5, Resources::from_bronze(100.), "n ^ 2"),
        (5, Resources::from_bronze(100.), "n ^ 2.5"),
        (5, Resources::from_bronze(100.), "n ^ 3"),
        (5, Resources::from_bronze(100.), "n ^ 4"),
        (5, Resources::from_bronze(100.), "n ^ 5"),
        (5, Resources::from_bronze(100.), "n ^ 6"),
        (5, Resources::from_bronze(100.), "n ^ 7"),
        (5, Resources::from_bronze(100.), "n ^ 8"),
        (5, Resources::from_bronze(100.), "n ^ 9"),
        (5, Resources::from_bronze(100.), "n ^ 10"),
    ]
);

// TODO
impl_upgrade!(
    MaxInstructions,
    type=u64,
    level=2,
    [
        (100, Resources::from_bronze(50.), "100"),
        (100, Resources::from_bronze(50.), "200"),
        (100, Resources::from_bronze(50.), "300"),
        (100, Resources::from_bronze(50.), "400"),
        (500, Resources::from_bronze(1e3), "500"),
        (500, Resources::from_bronze(1e3), "750"),
        (500, Resources::from_bronze(1e3), "1000"),
        (500, Resources::from_bronze(1e3), "1250"),
        (500, Resources::from_bronze(1e3), "1500"),
        (2000, Resources::from_bronze(50e3), "2000"),
        (2000, Resources::from_bronze(50e3), "2500"),
        (2000, Resources::from_bronze(50e3), "3000"),
        (2000, Resources::from_bronze(50e3), "4000"),
        (2000, Resources::from_bronze(50e3), "5000"),
        (2000, Resources::from_bronze(50e3), "6000"),
        (2000, Resources::from_bronze(50e3), "8000"),
        (10_000, Resources::from_bronze(5e6), "10000"),
        (100_000, Resources::from_bronze(5e6), "10000"),
        (1_000_000, Resources::from_bronze(5e6), "10000"),
        (100_000_000, Resources::from_bronze(5e6), "10000"),
        (1_000_000_000, Resources::zero(), "100000"),
    ]
);

impl_upgrade!(
    UnlockReboot,
    type=bool,
    level=2,
    [
        (false, Resources::from_silver(100.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    CodeExpressionLiterals,
    type=(),
    level=2,
    [
        ((), Resources::from_bronze(200.), "0, 1"),
        ((), Resources::from_bronze(200.), "2"),
        ((), Resources::from_bronze(200.), "3, 4, 5"),
        ((), Resources::from_bronze(200.), "6-10"),
        ((), Resources::zero(), "numbers to 100"),
        ((), Resources::zero(), "numbers to 255"),
        ((), Resources::zero(), "empty strings"),
    ]
);

impl_upgrade!(
    KeepPrestigeUpgrades,
    type=bool,
    level=2,
    [
        (false, Resources::from_bronze(500.), "keep L0"),
        (false, Resources::from_bronze(500.), "keep L1"),
        (false, Resources::from_bronze(500.), "keep L2"),
        (false, Resources::from_bronze(500.), "keep L3"),
        (false, Resources::from_bronze(500.), "keep L4"),
        (false, Resources::from_bronze(500.), "keep L5"),
        (false, Resources::zero(), "keep L6"),
    ]
);

impl_upgrade!(
    UnlockLevel3,
    type=bool,
    level=2,
    [
        (false, Resources::from_silver(1e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 3

impl_upgrade!(
    AutoCompile,
    type=bool,
    level=3,
    [
        (false, Resources::from_silver(5e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    UnlockPrint,
    type=bool,
    level=3,
    [
        (false, Resources::from_silver(10e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    PrintSpeedReset,
    type=f32,
    level=3,
    [
        (1.0, Resources::from_silver(20e3), "^0"),
        (0.5, Resources::from_silver(100e3), "^0.1"),
        (0.5, Resources::from_silver(100e3), "^0.2"),
        (0.5, Resources::from_silver(100e3), "^0.3"),
        (0.5, Resources::from_silver(100e3), "^0.4"),
        (0.5, Resources::from_silver(100e3), "^0.5"),
        (0.5, Resources::from_silver(100e3), "^0.6"),
        (0.5, Resources::from_silver(100e3), "^0.7"),
        (0.5, Resources::from_silver(100e3), "^0.8"),
        (0.5, Resources::from_silver(100e3), "^0.9"),
        (0.5, Resources::from_silver(100e3), "none"),
    ]
);

// TODO
impl_upgrade!(
    SilverPerPrintCharacter,
    type=(u32, u8),
    level=3,
    [
        ((0, 0), Resources::from_silver(50e3), "1"),
        ((0, 0), Resources::from_silver(50e3), "2"),
        ((0, 0), Resources::from_silver(50e3), "3"),
        ((0, 0), Resources::from_silver(50e3), "4"),
        ((0, 0), Resources::from_silver(50e3), "5"),
        ((1, 1), Resources::from_silver(50e3), "n"),
        ((2, 1), Resources::from_silver(500e3), "2n"),
        ((2, 1), Resources::from_silver(500e3), "3n"),
        ((2, 1), Resources::from_silver(500e3), "4n"),
        ((5, 1), Resources::from_gold(1.), "5n"),
        ((1, 2), Resources::zero(), "n^2"),
        ((1, 3), Resources::zero(), "n^3"),
        ((1, 5), Resources::zero(), "n^5"),
        ((1, 10), Resources::zero(), "n^10"),
    ]
);

impl_upgrade!(
    RessourcesAfterReboot,
    type=Resources,
    level=3,
    [
        (Resources::zero(), Resources::from_silver(100.), "0"),
        (Resources::new(100.0, 0.0, 0.0, 0.0, 0.0), Resources::from_gold(100.), format!("{}", Resources::new(100.0, 0.0, 0.0, 0.0, 0.0).fmt_oneline())),
        (Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0), Resources::from_diamond(100.), format!("{}", Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0).fmt_oneline())),
        (Resources::new(1e6, 10_000., 100., 0.0, 0.0), Resources::zero(), format!("{}", Resources::new(1e6, 10_000., 100., 0.0, 0.0).fmt_oneline())),
    ]
);

impl_upgrade!(
    UnlockLevel4,
    type=bool,
    level=3,
    [
        (false, Resources::from_gold(10.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 4

impl_upgrade!(
    UnlockSleep,
    type=bool,
    level=4,
    [
        (false, Resources::from_gold(50.), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// TODO
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

// TODO
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

// TODO
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
        (false, Resources::from_gold(10e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 5

impl_upgrade!(
    AutoRun,
    type=bool,
    level=5,
    [
        (false, Resources::from_gold(50e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

impl_upgrade!(
    UnlockBrk,
    type=bool,
    level=5,
    [
        (false, Resources::from_gold(100e3), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// TODO
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

// TODO
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
        (false, Resources::from_gold(100e6), LOCKED),
        (true, Resources::zero(), UNLOCKED),
    ]
);

// Level 6

impl_upgrade!(
    GainCurrencyFunction,
    type=bool,
    level=6,
    [
        (false, Resources::from_gold(1e9), LOCKED),
        (true, Resources::zero(), UNLOCKED),
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
