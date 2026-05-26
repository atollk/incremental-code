use crate::game_state::Resources;
use helper_macros::FieldsAs;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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
    /// Cost to advance from the current level to the next, or `None` if already maxed.
    fn next_level_cost(&self) -> Option<Resources>;

    /// Advance this upgrade by one level, capping at [`max_level`](Self::max_level).
    fn level_up(&mut self);
    /// Reduce this upgrade by one level, clamping at zero.
    fn level_down(&mut self);

    fn num_cost_tracks(&self) -> usize;
    fn track_get_level(&self, track: usize) -> u8;
    fn track_next_cost(&self, track: usize) -> Option<Resources>;
    fn track_level_up(&mut self, track: usize);
    fn track_level_down(&mut self, track: usize);

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
        values=[ $( ($value:expr, $text:expr) ),+ $(,)? ],
        costs=[ $( [ $( $cost:expr ),+ $(,)? ] ),+ $(,)? ]
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

            pub(crate) fn cost_at(level: u8) -> Option<Resources> {
                Self::cost_at_track(0, level)
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
                self.0
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len().saturating_sub(1) as u8
            }

            fn value_text(&self) -> Cow<'static, str> {
                Self::value_text_at(self.0).unwrap_or_else(|| self.fail_oob())
            }

            fn next_level_cost(&self) -> Option<Resources> {
                self.track_next_cost(0)
            }

            fn level_up(&mut self) {
                self.0 = std::cmp::min(self.0 + 1, self.max_level());
            }

            fn level_down(&mut self) {
                self.0 = self.0.saturating_sub(1);
            }

            fn num_cost_tracks(&self) -> usize {
                [ $( impl_upgrade!(@unit_track [ $( $cost ),+ ]) ),+ ].len()
            }

            fn track_get_level(&self, _track: usize) -> u8 {
                self.0
            }

            fn track_next_cost(&self, track: usize) -> Option<Resources> {
                Self::cost_at_track(track, self.0)
            }

            fn track_level_up(&mut self, _track: usize) {
                self.level_up()
            }

            fn track_level_down(&mut self, _track: usize) {
                self.level_down()
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
    type=u32,
    level=1,
    values=[
        (1, "100 %"),
        (1, "90 %"),
        (1, "80 %"),
        (1, "70 %"),
        (1, "60 %"),
        (1, "50 %"),
        (2, "40 %"),
        (2, "30 %"),
        (4, "25 %"),
        (4, "20 %"),
        (4, "15 %"),
        (10, "10 %"),
        (10, "5 %"),
        (10, "2.5 %"),
        (10, "1 %"),
        (10, "0.5 %"),
        (10, "0.25 %"),
        (10, "0.1 %"),
        (10, "n ^ -0.1"),
        (10, "n ^ -0.2"),
        (10, "n ^ -0.3"),
        (10, "n ^ -0.4"),
        (10, "n ^ -0.5"),
        (10, "n ^ -0.6"),
        (10, "n ^ -0.7"),
        (10, "n ^ -0.8"),
        (10, "n ^ -0.9"),
        (10, "n ^ -1"),
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
    type=u32,
    level=2,
    values=[
        (1, "1"),
        (1, "2"),
        (1, "3"),
        (1, "4"),
        (5, "5"),
        (5, "6"),
        (5, "7"),
        (5, "8"),
        (5, "9"),
        (5, "10"),
        (5, "15"),
        (5, "20"),
        (5, "25"),
        (5, "30"),
        (5, "35"),
        (5, "40"),
        (5, "50"),
        (5, "100"),
        (5, "n"),
        (5, "n ^ 1.5"),
        (5, "n ^ 2"),
        (5, "n ^ 2.5"),
        (5, "n ^ 3"),
        (5, "n ^ 4"),
        (5, "n ^ 5"),
        (5, "n ^ 6"),
        (5, "n ^ 7"),
        (5, "n ^ 8"),
        (5, "n ^ 9"),
        (5, "n ^ 10"),
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

// TODO
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
    type=(),
    level=2,
    values=[
        ((), "0, 1"),
        ((), "2"),
        ((), "3, 4, 5"),
        ((), "6-10"),
        ((), "numbers to 100"),
        ((), "numbers to 255"),
        ((), "empty strings"),
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
    type=bool,
    level=2,
    values=[
        (false, "keep L0"),
        (false, "keep L1"),
        (false, "keep L2"),
        (false, "keep L3"),
        (false, "keep L4"),
        (false, "keep L5"),
        (false, "keep L6"),
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
    UnlockPrint,
    type=bool,
    level=3,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_silver(10e3), Resources::zero()]]
);

impl_upgrade!(
    PrintSpeedReset,
    type=f32,
    level=3,
    values=[
        (1.0, "^0"),
        (0.5, "^0.1"),
        (0.5, "^0.2"),
        (0.5, "^0.3"),
        (0.5, "^0.4"),
        (0.5, "^0.5"),
        (0.5, "^0.6"),
        (0.5, "^0.7"),
        (0.5, "^0.8"),
        (0.5, "^0.9"),
        (0.5, "none"),
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

// TODO
impl_upgrade!(
    SilverPerPrintCharacter,
    type=(u32, u8),
    level=3,
    values=[
        ((0, 0), "1"),
        ((0, 0), "2"),
        ((0, 0), "3"),
        ((0, 0), "4"),
        ((0, 0), "5"),
        ((1, 1), "n"),
        ((2, 1), "2n"),
        ((2, 1), "3n"),
        ((2, 1), "4n"),
        ((5, 1), "5n"),
        ((1, 2), "n^2"),
        ((1, 3), "n^3"),
        ((1, 5), "n^5"),
        ((1, 10), "n^10"),
    ],
    costs=[[
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(50e3),
        Resources::from_silver(500e3),
        Resources::from_silver(500e3),
        Resources::from_silver(500e3),
        Resources::from_gold(1.),
        Resources::zero(),
        Resources::zero(),
        Resources::zero(),
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
    UnlockSleep,
    type=bool,
    level=4,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(50.), Resources::zero()]]
);

// TODO
impl_upgrade!(
    MinInstructionDuration,
    type=f32,
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

// TODO
impl_upgrade!(
    InstructionSpeedToSleep,
    type=f32,
    level=4,
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

// TODO
impl_upgrade!(
    GoldPerSleepSecond,
    type=f32,
    level=4,
    values=[
        (1.0, "1"),
        (5.0, "5"),
        (25.0, "25"),
    ],
    costs=[[
        Resources::from_gold(500.),
        Resources::from_gold(5e3),
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
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(50e3), Resources::zero()]]
);

impl_upgrade!(
    UnlockBrk,
    type=bool,
    level=5,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(100e3), Resources::zero()]]
);

// TODO
impl_upgrade!(
    BreakSlowdown,
    type=f32,
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

// TODO
impl_upgrade!(
    DiamondPerBreakPoint,
    type=f32,
    level=5,
    values=[
        (1.0, "1"),
        (5.0, "5"),
        (25.0, "25"),
    ],
    costs=[[
        Resources::from_gold(1e6),
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

impl_upgrade!(
    GainCurrencyFunction,
    type=bool,
    level=6,
    values=[(false, LOCKED), (true, UNLOCKED)],
    costs=[[Resources::from_gold(1e9), Resources::zero()]]
);

impl_upgrade!(
    WinCondition,
    type=bool,
    level=6,
    values=[(false, "not won"), (true, "won")],
    costs=[[Resources::from_gold(1e12), Resources::zero()]]
);

impl CodeExpressionLiterals {
    pub(crate) fn get_max_int_literal(&self) -> u8 {
        match self.0 {
            0 => 1,
            1 => 2,
            2 => 5,
            3 => 10,
            4 => 100,
            _ => 255,
        }
    }
}
