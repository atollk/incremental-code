use crate::game_state::Resources;
use helper_macros::FieldsAs;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::time::Duration;

/// Common interface for all purchasable upgrades.
pub trait Upgrade: dyn_clone::DynClone {
    fn name(&self) -> &'static str;
    /// The unlock tier this upgrade belongs to.
    fn group(&self) -> usize;
    /// The player's current level for this upgrade (0-based).
    fn get_level(&self) -> u8;
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Human-readable description of the current effect value.
    fn value_text(&self) -> Cow<'static, str>;
    /// Human-readable description of the next level effect value.
    fn next_level_value_text(&self) -> Option<Cow<'static, str>>;

    fn count_tracks(&self) -> usize;
    fn track_get_level(&self, track: usize) -> u8;
    fn track_next_cost(&self, track: usize) -> Option<Resources>;
    fn track_level_up(&mut self, track: usize);
    fn track_level_down(&mut self, track: usize);
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
    pub unlock_sleep: UnlockSleep,
    pub sleep_speed_reset: SleepSpeedReset,
    pub silver_per_sleep_second: SilverPerSleepSecond,
    pub resources_after_reboot: RessourcesAfterReboot,
    pub unlock_level4: UnlockLevel4,
    // Level 4
    pub unlock_print: UnlockPrint,
    pub min_instruction_duration: MinInstructionDuration,
    pub gold_per_print_character: GoldPerPrintCharacter,
    pub unlock_level5: UnlockLevel5,
    // Level 5
    pub auto_run: AutoRun,
    pub unlock_brk: UnlockBrk,
    pub brk_slowdown: BrkSlowdown,
    pub diamond_per_brk: DiamondPerBrk,
    pub unlock_level6: UnlockLevel6,
    // Level 6
    pub win_condition: WinCondition,
}

impl Upgrades {
    pub(crate) const UPGRADES_LEN: usize = 32;

    /// Returns all upgrades as an array of trait-object references.
    pub fn upgrades(&self) -> [&dyn Upgrade; Self::UPGRADES_LEN] {
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
        values=[ $( ($value:expr, $text:expr) ),+ $(,)? ],
        costs=[ $( [ $( $cost:expr ),+ $(,)? ] ),+ $(,)? ]
    ) => {
        #[derive(Debug, Clone, PartialEq, std::hash::Hash, serde::Serialize, serde::Deserialize)]
        pub(crate) struct $struct(
            [u8; { [ $( impl_upgrade!(@unit_track [ $( $cost ),+ ]) ),+ ].len() }]
        );

        impl Default for $struct {
            fn default() -> Self {
                Self([0u8; _])
            }
        }

        impl $struct {
            fn fail_oob(&self) -> ! {
                panic!(
                    concat!(stringify!($struct), ": level {} out of bounds"),
                    self.total_level()
                )
            }

            fn total_level(&self) -> u8 {
                self.0.iter().copied().map(u16::from).sum::<u16>() as u8
            }

            pub(crate) fn value_at(level: u8) -> Option<$val> {
                let mut __i: u8 = 0;
                $(
                    if level == __i { return Some($value); }
                    __i += 1;
                )+
                None
            }

            pub(crate) fn cost_at_track(track: usize, level: u8) -> Option<Resources> {
                let mut __t: usize = 0;
                $(
                    if track == __t {
                        let mut __i: u8 = 0;
                        $(
                            if level == __i { return Some($cost); }
                            __i += 1;
                        )+
                        return None;
                    }
                    __t += 1;
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
                Self::value_at(self.total_level()).unwrap_or_else(|| self.fail_oob())
            }
        }

        // Verify that `costs` and `values` match in size.
        const _: () = {
            let values_len = [ $( impl_upgrade!(@unit $value) ),+ ].len();
            $(
                let costs_len = [ $( impl_upgrade!(@unit $cost) ),+ ].len();
                assert!(
                    values_len == costs_len,
                    concat!(stringify!($struct), ": cost track length must equal values length"),
                );
            )+
        };

        impl Upgrade for $struct {
            fn name(&self) -> &'static str {
                stringify!($struct)
            }

            fn group(&self) -> usize {
                $group_level
            }

            fn get_level(&self) -> u8 {
                self.total_level()
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len().saturating_sub(1) as u8
            }

            fn value_text(&self) -> Cow<'static, str> {
                Self::value_text_at(self.total_level()).unwrap_or_else(|| self.fail_oob())
            }

            fn next_level_value_text(&self) -> Option<Cow<'static, str>> {
                Self::value_text_at(self.total_level() + 1)
            }

            fn count_tracks(&self) -> usize {
                self.0.len()
            }

            fn track_get_level(&self, track: usize) -> u8 {
                self.0[track]
            }

            fn track_next_cost(&self, track: usize) -> Option<Resources> {
                let total = self.total_level();
                if total >= self.max_level() {
                    return None;
                }
                Self::cost_at_track(track, total)
            }

            fn track_level_up(&mut self, track: usize) {
                if self.total_level() < self.max_level() {
                    self.0[track] += 1;
                }
            }

            fn track_level_down(&mut self, track: usize) {
                self.0[track] = self.0[track].saturating_sub(1);
            }
        }
    };
    (@unit $_:expr) => { () };
    (@unit_track [ $( $_track_cost:expr ),+ ]) => { () };
}

const LOCKED: &str = "🔒";
const UNLOCKED: &str = "🔓";

// Level 0

impl_upgrade!(
    UnlockCode,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_bronze(0.), Resources::zero()]]
);

impl_upgrade!(
    UnlockHud,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_bronze(1.), Resources::zero()]]
);

impl_upgrade!(
    UnlockMusic,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_bronze(5.), Resources::zero()]]
);

impl_upgrade!(
    UnlockLevel1,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_bronze(5.), Resources::zero()]]
);

// Level 1

impl_upgrade!(
    CompileTime,
    type=f32,
    level=1,
    values=[
        (10., "10s"),
        (5., "5s"),
        (4., "4s"),
        (3., "3s"),
        (2., "2s"),
        (1., "1s"),
        (0.1, "0.1s"),
    ],
    costs=[[
        Resources::from_bronze(10.),
        Resources::from_bronze(100.),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e6),
        Resources::from_bronze(1e9),
        Resources::from_silver(10.),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    InstructionExecutionSpeed,
    type=(f64, f64),  // constant, exponent
    level=1,
    values=[
        ((1., 0.), "100 %"),
        ((0.9, 0.), "90 %"),
        ((0.8, 0.), "80 %"),
        ((0.7, 0.), "70 %"),
        ((0.6, 0.), "60 %"),
        ((0.5, 0.), "50 %"),
        ((0.4, 0.), "40 %"),
        ((0.3, 0.), "30 %"),
        ((0.25, 0.), "25 %"),
        ((0.2, 0.), "20 %"),
        ((0.15, 0.), "15 %"),
        ((0.1, 0.), "10 %"),
        ((0.05, 0.), "5 %"),
        ((0.025, 0.), "2.5 %"),
        ((0.01, 0.), "1 %"),
        ((0.005, 0.), "0.5 %"),
        ((0.0025, 0.), "0.25 %"),
        ((0.001, 0.), "0.1 %"),
        ((0.001, -0.1), "n ^ -0.1"),
        ((0.001, -0.2), "n ^ -0.2"),
        ((0.001, -0.3), "n ^ -0.3"),
        ((0.001, -0.4), "n ^ -0.4"),
        ((0.001, -0.5), "n ^ -0.5"),
        ((0.001, -0.6), "n ^ -0.6"),
        ((0.001, -0.7), "n ^ -0.7"),
        ((0.001, -0.8), "n ^ -0.8"),
        ((0.001, -0.9), "n ^ -0.9"),
        ((0.001, -1.), "n ^ -1"),
    ],
    costs=[[
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
        Resources::from_bronze(30e3),
    ]]
);

impl_upgrade!(
    CodeLineWidth,
    type=u8,
    level=1,
    values=[
        (5, "5"),
        (10, "10"),
        (15, "15"),
        (30, "30"),
        (50, "50"),
        (80, "80"),
    ],
    costs=[[
        Resources::from_bronze(5.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100e3),
        Resources::from_bronze(10e6),
        Resources::from_silver(100.),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    CodeLineCount,
    type=u8,
    level=1,
    values=[
        (1, "1"),
        (2, "2"),
        (4, "4"),
        (4, "5"),
        (6, "6"),
        (6, "7"),
        (8, "8"),
        (10, "10"),
        (10, "15"),
        (20, "20"),
        (30, "30"),
        (40, "40"),
    ],
    costs=[[
        Resources::from_bronze(5.),
        Resources::from_bronze(25.),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e3),
        Resources::from_bronze(100e3),
        Resources::from_bronze(100e3),
        Resources::from_bronze(10e6),
        Resources::from_silver(10.),
        Resources::from_silver(10.),
        Resources::from_silver(1e3),
        Resources::from_gold(100.),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockLevel2,
    type=bool,
    level=1,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_bronze(100e3), Resources::zero()]]
);

// Level 2

impl_upgrade!(
    BronzePerInstruction,
    type=(u8, f32),  // constant, exponent
    level=2,
    values=[
        ((1, 0.), "1"),
        ((2, 0.), "2"),
        ((3, 0.), "3"),
        ((4, 0.), "4"),
        ((5, 0.), "5"),
        ((6, 0.), "6"),
        ((7, 0.), "7"),
        ((8, 0.), "8"),
        ((9, 0.), "9"),
        ((10, 0.), "10"),
        ((15, 0.), "15"),
        ((20, 0.), "20"),
        ((25, 0.), "25"),
        ((30, 0.), "30"),
        ((35, 0.), "35"),
        ((40, 0.), "40"),
        ((50, 0.), "50"),
        ((100, 0.), "100"),
        ((100, 1.), "n"),
        ((100, 1.5), "n ^ 1.5"),
        ((100, 2.), "n ^ 2"),
        ((100, 2.5), "n ^ 2.5"),
        ((100, 3.), "n ^ 3"),
        ((100, 4.), "n ^ 4"),
        ((100, 5.), "n ^ 5"),
        ((100, 6.), "n ^ 6"),
        ((100, 7.), "n ^ 7"),
        ((100, 8.), "n ^ 8"),
        ((100, 9.), "n ^ 9"),
        ((100, 10.), "n ^ 10"),
    ],
    costs=[[
        Resources::from_bronze(10.),
        Resources::from_bronze(10.),
        Resources::from_bronze(10.),
        Resources::from_bronze(10.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
        Resources::from_bronze(100.),
    ]]
);

impl_upgrade!(
    MaxInstructions,
    type=u64,
    level=2,
    values=[
        (100, "100"),
        (100, "200"),
        (100, "300"),
        (100, "400"),
        (500, "500"),
        (500, "750"),
        (500, "1000"),
        (500, "1250"),
        (500, "1500"),
        (2000, "2000"),
        (2000, "2500"),
        (2000, "3000"),
        (2000, "4000"),
        (2000, "5000"),
        (2000, "6000"),
        (2000, "8000"),
        (10_000, "10000"),
        (100_000, "10000"),
        (1_000_000, "10000"),
        (100_000_000, "10000"),
        (1_000_000_000, "100000"),
    ],
    costs=[[
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(50.),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e3),
        Resources::from_bronze(1e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(50e3),
        Resources::from_bronze(5e6),
        Resources::from_bronze(5e6),
        Resources::from_bronze(5e6),
        Resources::from_bronze(5e6),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockReboot,
    type=bool,
    level=2,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_silver(100.), Resources::zero()]]
);

impl_upgrade!(
    CodeExpressionLiterals,
    type=(bool, u8),  // strings unlocked ; max int
    level=2,
    values=[
        ((false, 1), "0, 1"),
        ((false, 2), "2"),
        ((false, 5), "3, 4, 5"),
        ((false, 10), "6-10"),
        ((true, 10), "empty strings"),
        ((true, 100), "numbers to 100"),
        ((true, 255), "numbers to 255"),
    ],
    costs=[[
        Resources::from_bronze(200.),
        Resources::from_bronze(200.),
        Resources::from_bronze(200.),
        Resources::from_bronze(200.),
        Resources::zero(),
        Resources::zero(),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    KeepPrestigeUpgrades,
    type=u8,
    level=2,
    values=[
        (0, "keep L0"),
        (1, "keep L1"),
        (2, "keep L2"),
        (3, "keep L3"),
        (4, "keep L4"),
        (5, "keep L5"),
        (6, "keep L6"),
    ],
    costs=[[
        Resources::from_bronze(500.),
        Resources::from_bronze(500.),
        Resources::from_bronze(500.),
        Resources::from_bronze(500.),
        Resources::from_bronze(500.),
        Resources::from_bronze(500.),
        Resources::zero(),
    ]]
);

pub enum CodeStatementLevels {
    None,
    SimpleLoops,
    NestedLoops,
    Functions,
    SingleRecursion,
    MultiRecursion,
}

impl_upgrade!(
    CodeStatements,
    type=CodeStatementLevels,
    level=2,
    values=[
        (CodeStatementLevels::None, ""),
        (CodeStatementLevels::SimpleLoops, "simple loops"),
        (CodeStatementLevels::NestedLoops, "nested loops"),
        (CodeStatementLevels::Functions, "functions"),
        (CodeStatementLevels::SingleRecursion, "single recursion"),
        (CodeStatementLevels::MultiRecursion, "multi recursion"),
    ],
    costs=[[
        Resources::from_bronze(500e3),
        Resources::from_bronze(500e3),
        Resources::from_bronze(500e3),
        Resources::from_bronze(500e3),
        Resources::from_bronze(500e3),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockLevel3,
    type=bool,
    level=2,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_silver(1e3), Resources::zero()]]
);

// Level 3

impl_upgrade!(
    AutoCompile,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_silver(5e3), Resources::zero()]]
);

impl_upgrade!(
    UnlockSleep,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(50.), Resources::zero()]]
);

/*
impl_upgrade!(
    InstructionSpeedToSleep,
    type=f32,
    level=3,
    values=[
        (1.0, "1x"),
        (2.0, "2x"),
        (5.0, "5x"),
    ],
    costs=[[
        Resources::from_gold(200.),
        Resources::from_gold(2e3),
        Resources::zero(),
    ]]
);
 */

impl_upgrade!(
    SleepSpeedReset,
    type=f64,
    level=3,
    values=[
        (0., "^0"),
        (0.1, "^0.1"),
        (0.2, "^0.2"),
        (0.3, "^0.3"),
        (0.4, "^0.4"),
        (0.5, "^0.5"),
        (0.6, "^0.6"),
        (0.7, "^0.7"),
        (0.8, "^0.8"),
        (0.9, "^0.9"),
        (1.0, "none"),
    ],
    costs=[[
        Resources::from_silver(20e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::from_silver(100e3),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    SilverPerSleepSecond,
    type=u8,  // linear
    level=3,
    values=[
        (1, "1"),
        (2, "2"),
        (3, "3"),
        (5, "5"),
        (10, "10"),
    ],
    costs=[[
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    RessourcesAfterReboot,
    type=Resources,
    level=3,
    values=[
        (Resources::zero(), "0"),
        (Resources::new(100.0, 0.0, 0.0, 0.0, 0.0), format!("{}", Resources::new(100.0, 0.0, 0.0, 0.0, 0.0).fmt_oneline())),
        (Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0), format!("{}", Resources::new(10_000.0, 100.0, 0.0, 0.0, 0.0).fmt_oneline())),
        (Resources::new(1e6, 10_000., 100., 0.0, 0.0), format!("{}", Resources::new(1e6, 10_000., 100., 0.0, 0.0).fmt_oneline())),
    ],
    costs=[[
        Resources::from_silver(100.),
        Resources::from_gold(100.),
        Resources::from_diamond(100.),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockLevel4,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(10.), Resources::zero()]]
);

// Level 4

impl_upgrade!(
    UnlockPrint,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_silver(10e3), Resources::zero()]]
);

impl_upgrade!(
    MinInstructionDuration,
    type=f64,
    level=4,
    values=[
        (1.0, "1ms"),
        (0.1, "0.1ms"),
        (0.01, "0.01ms"),
    ],
    costs=[[
        Resources::from_gold(100.),
        Resources::from_gold(1e3),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    GoldPerPrintCharacter,
    type=u8,
    level=4,
    values=[
        (100, "1"),
        (3, "log(log(log(n)))"),
        (2, "log(log(n))"),
        (1, "log(n)"),
    ],
    costs=[[
        Resources::from_gold(500.),
        Resources::from_gold(5e3),
        Resources::zero(),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockLevel5,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(10e3), Resources::zero()]]
);

// Level 5

impl_upgrade!(
    AutoRun,
    type=Option<Duration>,
    level=5,
    values=[
        (None, LOCKED),
        (Some(Duration::from_secs(18)), "18s"),
        (Some(Duration::from_secs(6)), "6s"),
        (Some(Duration::from_secs(2)), "2s"),
        (Some(Duration::from_secs(1)), "0s"),
    ],
    costs=[[
        Resources::from_gold(50e3),
        Resources::zero(),
        Resources::zero(),
        Resources::zero(),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockBrk,
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(100e3), Resources::zero()]]
);

impl_upgrade!(
    BrkSlowdown,
    type=f64,
    level=5,
    values=[
        (2.0, "2x"),
        (5.0, "5x"),
        (10.0, "10x"),
    ],
    costs=[[
        Resources::from_gold(500e3),
        Resources::from_gold(5e6),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    DiamondPerBrk,
    type=u16,  // linear
    level=5,
    values=[
        (1, "1"),
        (3, "3"),
        (5, "5"),
        (25, "25"),
        (1000, "1000"),
    ],
    costs=[[
        Resources::from_gold(1e6),
        Resources::from_gold(10e6),
        Resources::from_gold(10e6),
        Resources::from_gold(10e6),
        Resources::zero(),
    ]]
);

impl_upgrade!(
    UnlockLevel6,
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(100e6), Resources::zero()]]
);

// Level 6

/*
impl_upgrade!(
    GainCurrencyFunction,
    type=bool,
    level=6,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(1e9), Resources::zero()]]
);
 */

impl_upgrade!(
    WinCondition,
    type=bool,
    level=6,
    values=[(false, "not won"), (true, "won")],
    costs=[[Resources::from_gold(1e12), Resources::zero()]]
);
