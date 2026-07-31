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
    /// The highest level this upgrade can reach.
    fn max_level(&self) -> u8;
    /// Human-readable description of the current effect value.
    fn value_text(&self) -> Cow<'static, str>;
    /// Human-readable description of the next level effect value.
    fn next_level_value_text(&self) -> Option<Cow<'static, str>>;

    fn count_tracks(&self) -> usize;
    fn track_get_level(&self, track: usize) -> u8;
    /// The highest level this specific track can reach on its own.
    fn track_max_level(&self, track: usize) -> u8;
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
    pub auto_run: AutoRun,
    pub unlock_print: UnlockPrint,
    pub min_instruction_duration: MinInstructionDuration,
    pub keep_code_on_prestige: KeepCodeOnPrestige,
    pub gold_print_log_nesting: GoldPrintLogNesting,
    pub unlock_level5: UnlockLevel5,
    // Level 5
    pub unlock_brk: UnlockBrk,
    pub brk_slowdown: BrkSlowdown,
    pub diamond_per_brk: DiamondPerBrk,
    pub unlock_level6: UnlockLevel6,
    // Level 6
    pub additive_reboot: AdditiveReboot,
    pub win_condition: WinCondition,
}

impl Upgrades {
    pub(crate) const UPGRADES_LEN: usize = 34;

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

            pub(crate) fn track_len(track: usize) -> u8 {
                let mut __t: usize = 0;
                $(
                    if track == __t {
                        return [ $( impl_upgrade!(@unit $cost) ),+ ].len() as u8;
                    }
                    __t += 1;
                )+
                0
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

        // Verify that the sum of all track cost lengths equals values_len - 1.
        const _: () = {
            let values_len = [ $( impl_upgrade!(@unit $value) ),+ ].len();
            let mut sum_tracks_len: usize = 0;
            $(
                sum_tracks_len += [ $( impl_upgrade!(@unit $cost) ),+ ].len();
            )+
            assert!(
                sum_tracks_len == values_len - 1,
                concat!(stringify!($struct), ": sum of cost track lengths must equal values length - 1"),
            );
        };

        impl Upgrade for $struct {
            fn name(&self) -> &'static str {
                stringify!($struct)
            }

            fn group(&self) -> usize {
                $group_level
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

            fn track_max_level(&self, track: usize) -> u8 {
                Self::track_len(track)
            }

            fn track_next_cost(&self, track: usize) -> Option<Resources> {
                let total = self.total_level();
                if total >= self.max_level() {
                    return None;
                }
                Self::cost_at_track(track, self.track_get_level(track))
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
const UNLOCKED: &str = "☑️";

// Level 0

impl_upgrade!(
    UnlockCode,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[
        Resources::bronze(0)
    ]]
);

impl_upgrade!(
    UnlockHud,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[
        Resources::bronze(1)
    ]]
);

impl_upgrade!(
    UnlockMusic,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[
        Resources::bronze(5)
    ]]
);

impl_upgrade!(
    UnlockLevel1,
    type=bool,
    level=0,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[
        Resources::bronze(5)
    ]]
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
        Resources::bronze(10),
        Resources::bronze(100),
        Resources::bronze(1e3),
        Resources::silver(200),
        Resources::new(20e3, 1e3, 0, 0, 0),
        Resources::new(1000, 100, 10, 1, 0),
    ]]
);

impl_upgrade!(
    InstructionExecutionSpeed,
    type=(f64, f64),  // constant, exponent
    level=1,
    values=[
        ((1., 0.), "100 %"),
        ((0.8, 0.), "80 %"),
        ((0.6, 0.), "60 %"),
        ((0.4, 0.), "40 %"),
        ((0.3, 0.), "30 %"),
        ((0.15, 0.), "15 %"),
        ((0.05, 0.), "5 %"),
        ((0.025, 0.), "2.5 %"),
        ((0.01, 0.), "1 %"),
        ((0.0025, 0.), "0.25 %"),
        ((1e-3, 0.), "0.1 %"),
        ((1e-3, -0.1), "0.1% n ^ -0.1"),
        ((1e-3, -0.2), "0.1% n ^ -0.2"),
        ((1e-3, -0.3), "0.1% n ^ -0.3"),
        ((1e-3, -0.4), "0.1% n ^ -0.4"),
        ((1e-3, -0.5), "0.1% n ^ -0.5"),
        ((1e-3, -0.6), "0.1% n ^ -0.6"),
        ((1e-3, -0.7), "0.1% n ^ -0.7"),
        ((1e-3, -0.8), "0.1% n ^ -0.8"),
        ((1e-4, -0.9), "0.01% n ^ -0.9"),
        ((1e-4, -0.95), "0.01% n ^ -0.95"),
        ((1e-5, -0.99), "0.001% n ^ -0.99"),
        ((1e-6, -0.9999), "0.0001% n ^ -0.9999"),
    ],
    costs=[
        [
            Resources::bronze(10),
            Resources::bronze(20),
            Resources::bronze(30),
            Resources::bronze(200),
            Resources::bronze(1000),
            Resources::bronze(10e3),
            Resources::bronze(500e6),
        ],
        [
            Resources::silver(1),
            Resources::silver(5),
            Resources::silver(100),
            Resources::silver(2e3),
            Resources::silver(20e3),
            Resources::silver(900e21),
        ],
        [
            Resources::gold(30),
            Resources::gold(200),
            Resources::gold(10e3),
            Resources::gold(5e6),
            Resources::gold(100e21),
        ],
        [
            Resources::diamond(100e3),
            Resources::stars(10),
            Resources::diamond(1e9),
            Resources::stars(1000),
        ]
    ]
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
        Resources::bronze(100), // core upgrade 4
        Resources::silver(200), // core upgrade 13
        Resources::gold(300), // core upgrade 19
        Resources::diamond(2e3), // core upgrade 22
        Resources::silver(400e9) + Resources::gold(15e6), // core upgrade 24
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
        (5, "5"),
        (6, "6"),
        (7, "7"),
        (8, "8"),
        (10, "10"),
        (15, "15"),
        (20, "20"),
        (30, "30"),
        (40, "40"),
    ],
    costs=[[
        Resources::bronze(5), // core upgrade 1
        Resources::bronze(25), // core upgrade 2
        Resources::bronze(250), // core upgrade 5
        Resources::bronze(1e3), // core upgrade 6
        Resources::bronze(2e3), // core upgrade 8
        Resources::bronze(4e3), // core upgrade 10
        Resources::silver(300) + Resources::bronze(30e3), // core upgrade 15
        Resources::silver(1200) + Resources::bronze(70e3), // core upgrade 16
        Resources::silver(700) + Resources::bronze(70e3), // core upgrade 17
        Resources::diamond(2e3), // core upgrade 21
        Resources::diamond(10e3), // core upgrade 23
    ]]
);

impl_upgrade!(
    UnlockLevel2,
    type=bool,
    level=1,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::bronze(50)]]
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
        ((100, 5.), "n ^ 5"),
        ((100, 7.), "n ^ 7"),
        ((100, 10.), "n ^ 10"),
        ((100, 30.), "n ^ 30"),
    ],
    costs=[
        [
            Resources::bronze(20),
            Resources::bronze(30),
            Resources::bronze(40),
            Resources::bronze(50),
            Resources::bronze(70),
            Resources::bronze(90),
            Resources::bronze(120),
            Resources::bronze(150),
            Resources::bronze(180),
            Resources::bronze(400),
            Resources::bronze(1e3),
            Resources::bronze(4e3),
            Resources::bronze(10e3),
            Resources::bronze(1e9),
            Resources::bronze(1e15),
        ],
        [
            Resources::silver(1),
            Resources::silver(1),
            Resources::silver(10),
            Resources::silver(10e3),
            Resources::gold(50) + Resources::silver(100e3),
            Resources::diamond(300),
            Resources::gold(1.5e6),
            Resources::diamond(20e3),
            Resources::stars(10),
            Resources::gold(200e18),
            Resources::silver(1e120),
        ]
    ]
);

impl_upgrade!(
    MaxInstructions,
    type=u64,
    level=2,
    values=[
        (100, "100"),
        (1000, "1000"),
        (10_000, "10 k"),
        (100_000, "100 k"),
        (10_000_000, "10 M"),
        (1_000_000_000, "1 B"),
        (1_000_000_000_000, "1 T"),
    ],
    costs=[[
        Resources::silver(1000) + Resources::bronze(50e3),
        Resources::new(1e6, 1e3, 10, 0, 0),
        Resources::new(300e6, 100e3, 10e3, 0, 0),
        Resources::diamond(20),
        Resources::stars(10),
        Resources::bronze(1e120),
    ]]
);

impl_upgrade!(
    UnlockReboot,
    type=bool,
    level=2,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::bronze(500)]]
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
        Resources::bronze(1e3), // core upgrade 7
        Resources::bronze(3e3), // core upgrade 9
        Resources::bronze(10e3), // core upgrade 12
        Resources::silver(1e3), // core upgrade 12.5
        Resources::gold(100), // core upgrade 14
        Resources::bronze(600e12) + Resources::diamond(150e3), // core upgrade 27
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
        (6, "keep L5 & L6"),
    ],
    costs=[[
        Resources::silver(1),
        Resources::silver(10),
        Resources::diamond(1),
        Resources::new(1e9, 1e6, 1e3, 100, 0),
        Resources::stars(20),
    ]]
);

pub enum CodeStatementLevels {
    None,
    SimpleLoops,
    NestedLoops,
    Functions,
    Multiplication,
    PureFunctions,
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
        (CodeStatementLevels::Multiplication, "multiplication operations"),
        (CodeStatementLevels::PureFunctions, "pure functions"),
        (CodeStatementLevels::SingleRecursion, "single recursion"),
        (CodeStatementLevels::MultiRecursion, "multi recursion"),
    ],
    costs=[[
        Resources::silver(13), // core upgrade 11
        Resources::gold(10), // core upgrade 18
        Resources::bronze(100e6), // core upgrade 20
        Resources::gold(1e6),
        Resources::stars(10),
        Resources::stars(10), // core upgrade 25
        Resources::stars(5e3), // core upgrade 26
    ]]
);

impl_upgrade!(
    UnlockLevel3,
    type=bool,
    level=2,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::bronze(10e3)]]
);

// Level 3

impl_upgrade!(
    AutoCompile,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::silver(5e3)]]
);

impl_upgrade!(
    UnlockSleep,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::new(1., 1., 0., 0., 0.)]]
);

impl_upgrade!(
    SleepSpeedReset,
    type=f64,
    level=3,
    values=[
        (0., "^0"),
        (0.1, "^0.1"),
        (0.3, "^0.3"),
        (0.5, "^0.5"),
        (0.75, "^0.75"),
        (0.85, "^0.85"),
        (1.0, "none"),
    ],
    costs=[[
        Resources::bronze(80e3),
        Resources::bronze(200e3),
        Resources::bronze(300e6) + Resources::gold(1000),
        Resources::gold(11e3),
        Resources::diamond(400e3),
        Resources::bronze(40e48),
    ]]
);

impl_upgrade!(
    SilverPerSleepSecond,
    type=fn(f64) -> f64,  // linear
    level=3,
    values=[
        (|x| x.sqrt(), "sqrt(n)"),
        (|x| x.sqrt() * 2., "2 sqrt(n)"),
        (|x| x.sqrt() * 8., "8 sqrt(n)"),
        (|x| x.sqrt() * 128., "128 sqrt(n)"),
        (|x| x.sqrt() * 1e6, "1,000,000 sqrt(n)"),
        (|x| x, "n"),
        (|x| x.powi(8), "n^8"),
        (|x| x.powi(64), "n^64"),
    ],
    costs=[[
        Resources::bronze(100e3),
        Resources::gold(40) + Resources::bronze(300e3),
        Resources::gold(2e6),
        Resources::bronze(1e12),
        Resources::diamond(800e6),
        Resources::gold(10e12),
        Resources::stars(987),
    ]]
);

impl_upgrade!(
    RessourcesAfterReboot,
    type=Resources,
    level=3,
    values=[
        (Resources::zero(), "0"),
        (Resources::new(1000, 0, 0, 0, 0), format!("{}", Resources::new(1000, 0, 0, 0, 0).fmt_oneline())),
        (Resources::new(50_000, 1000, 0, 0, 0), format!("{}", Resources::new(50_000, 1000, 0, 0, 0).fmt_oneline())),
        (Resources::new(1e6, 10_000., 100., 0.0, 0.0), format!("{}", Resources::new(1e6, 10_000., 100., 0.0, 0.0).fmt_oneline())),
    ],
    costs=[[
        Resources::silver(100),
        Resources::gold(100),
        Resources::diamond(100),
    ]]
);

impl_upgrade!(
    UnlockLevel4,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::gold(12)]]
);

// Level 4

impl_upgrade!(
    AutoRun,
    type=Option<Duration>,
    level=4,
    values=[
        (None, LOCKED),
        (Some(Duration::from_secs(18)), "18s"),
        (Some(Duration::from_secs(6)), "6s"),
        (Some(Duration::from_secs(2)), "2s"),
        (Some(Duration::from_secs(1)), "1s"),
        (Some(Duration::from_millis(100)), "0.1s"),
    ],
    costs=[[
        Resources::gold(1),
        Resources::gold(100),
        Resources::diamond(10e3),
        Resources::bronze(1e15),
        Resources::stars(40),
    ]]
);

impl_upgrade!(
    UnlockPrint,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::new(1, 1, 1, 0, 0)]]
);

impl_upgrade!(
    MinInstructionDuration,
    type=f64,
    level=4,
    values=[
        (100e-6, "100,000 ns"),
        (10e-6, "10,000 ns"),
        (1e-6, "1000 ns"),
        (10e-9, "10 ns"),
        (10e-12, "0.01 ns"),
    ],
    costs=[[
        Resources::gold(1e6),
        Resources::new(100e12, 50e6, 1e6, 100, 0),
        Resources::stars(30),
        Resources::stars(10e3),
    ]]
);

impl_upgrade!(
    KeepCodeOnPrestige,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::gold(1e3)]]
);

fn iterated_log2_nonneg(mut x: f64, n: usize) -> f64 {
    for _ in 0..n {
        x = x.log2();
    }
    if x.is_finite() && x.is_sign_positive() {
        x.ceil()
    } else {
        0.
    }
}

impl_upgrade!(
    GoldPrintLogNesting,
    type=fn (f64) -> f64,
    level=4,
    values=[
        (|_| 1., "1"),
        (|x| iterated_log2_nonneg(x, 3), "log(log(log(n)))"),
        (|x| iterated_log2_nonneg(x, 2), "log(log(n))"),
        (|x| iterated_log2_nonneg(x, 1), "log(n)"),
        (|x| x.log2().powi(2), "log(n)^2"),
        (|x| x.log2().powi(4), "log(n)^4"),
        (|x| x.log2().powi(16), "log(n)^16"),
    ],
    costs=[[
        Resources::silver(5e3) + Resources::bronze(150e3),
        Resources::silver(25e3) + Resources::bronze(300e6),
        Resources::silver(100e3) + Resources::bronze(100e6),
        Resources::bronze(10e9) + Resources::diamond(100),
        Resources::bronze(20e27) + Resources::stars(10),
        Resources::silver(100e21),
    ]]
);

impl_upgrade!(
    UnlockLevel5,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::new(100e6, 100e3, 100, 10, 0)]]
);

// Level 5

impl_upgrade!(
    UnlockBrk,
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::new(1, 1, 1, 1, 0)]]
);

impl_upgrade!(
    BrkSlowdown,
    type=f64,
    level=5,
    values=[
        (0.1, "10%"),
        (0.2, "20%"),
        (0.5, "50%"),
    ],
    costs=[[
        Resources::bronze(20e15) + Resources::silver(1e12) + Resources::gold(50e6),
        Resources::diamond(150e9),
    ]]
);

impl_upgrade!(
    DiamondPerBrk,
    type=fn(f64) -> f64,  // linear
    level=5,
    values=[
        (|x| x, "1"),
        (|x| x * 100., "100"),
        (|x| x * 1e6, "1,000,000"),
        (|x| x * x * 1e6, "n^2"),
    ],
    costs=[[
        Resources::bronze(255e12),
        Resources::bronze(10e21),
        Resources::bronze(700e54),
    ]]
);

impl_upgrade!(
    UnlockLevel6,
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::stars(30)]]
);

// Level 6

impl_upgrade!(
    AdditiveReboot,
    type=(bool, bool),
    level=6,
    values=[
        ((false, false), LOCKED),
        ((true, false), UNLOCKED),
        ((true, true), "Reboot on Autorun"),
    ],
    costs=[[
        Resources::new(10e54, 10e21, 10e12, 10e9, 10),
        Resources::new(1e180, 1e130, 1e45, 10e12, 1000),
    ]]
);

/*
impl_upgrade!(
    GainCurrencyFunction,
    type=bool,
    level=6,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::inf()]]
);
 */

impl_upgrade!(
    WinCondition,
    type=bool,
    level=6,
    values=[(false, "not won"), (true, "won")],
    costs=[[Resources::inf()]]
);
