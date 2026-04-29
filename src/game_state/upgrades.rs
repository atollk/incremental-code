use crate::game_state::Resources;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

pub trait UpgradeCollection {
    fn upgrades(&self) -> impl Iterator<Item = &dyn Upgrade>;
    fn upgrades_mut(&mut self) -> impl Iterator<Item = &mut dyn Upgrade>;
}

pub trait Upgrade {
    fn name(&self) -> &'static str;
    fn current_level(&self) -> u8;
    fn max_level(&self) -> u8;
    fn next_level_cost(&self) -> Option<Resources>;
    fn current_value_text(&self) -> String;
    fn next_value_text(&self) -> Option<String>;

    fn level_up(&mut self);
    fn level_down(&mut self);

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

    fn format_cost_str(&self) -> String {
        match self.next_level_cost() {
            Some(r) => r.to_string(),
            None => "maxed".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Upgrades {
    pub level1: level1::Upgrades,
}

impl Default for Upgrades {
    fn default() -> Self {
        Upgrades {
            level1: level1::Upgrades::default(),
        }
    }
}

macro_rules! impl_upgrade {
    (
        $struct:ident,
        $val:ty,
        [ $( ($value:expr, $cost:expr) ),+ $(,)? ]
    ) => {
        #[derive(Debug, Default, Clone, std::hash::Hash, serde::Serialize, serde::Deserialize)]
        struct $struct (u8);

        impl $struct {
            #[allow(unused_assignments)]
            fn current_value_text(&self) -> $val {
                let mut __i: u8 = 0;
                $(
                    if self.0 == __i { return $value; }
                    __i += 1;
                )+
                panic!(
                    concat!(stringify!($struct), ": level {} out of bounds"),
                    self.0
                )
            }
        }

        impl Upgrade for $struct {
            fn name(&self) -> &'static str {
                stringify!($struct)
            }

            fn current_level(&self) -> u8 {
                self.0
            }

            fn max_level(&self) -> u8 {
                [ $( impl_upgrade!(@unit $value) ),+ ].len() as u8
            }

            #[allow(unused_assignments)]
            fn current_value_text(&self) -> String {
                let mut __i: u8 = 0;
                $(
                    if self.0 == __i { return format!("{}", $value); }
                    __i += 1;
                )+
                panic!(
                    concat!(stringify!($struct), ": level {} out of bounds"),
                    self.0
                )
            }

            #[allow(unused_assignments)]
            fn next_value_text(&self) -> Option<String> {
                let __target = self.0.checked_add(1)?;
                if __target >= [ $( impl_upgrade!(@unit $value) ),+ ].len() as u8 {
                    return None;
                }
                let mut __i: u8 = 0;
                $(
                    if __target == __i { return Some(format!("{}", $value)); }
                    __i += 1;
                )+
                None
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

pub mod level1 {
    use crate::game_state::Resources;
    use crate::game_state::upgrades::{Upgrade, UpgradeCollection};
    use serde::{Deserialize, Serialize};

    impl_upgrade!(
        CompileTime,
        f32,
        [
            (1.0, Resources::from_bronze(10.)),
            (0.9, Resources::from_bronze(20.)),
            (0.8, Resources::default()),
        ]
    );

    impl_upgrade!(
        RunTime,
        f32,
        [
            (1.0, Resources::from_bronze(10.)),
            (0.9, Resources::from_bronze(20.)),
            (0.8, Resources::default()),
        ]
    );

    impl_upgrade!(
        SpeedUpPerInstruction,
        f64,
        [
            (1.0, Resources::from_bronze(10.)),
            (1.01, Resources::from_bronze(20.)),
            (1.1, Resources::default()),
        ]
    );

    impl_upgrade!(
        BronzePerInstruction,
        u32,
        [
            (1, Resources::from_bronze(10.)),
            (2, Resources::from_bronze(20.)),
            (5, Resources::default()),
        ]
    );
    impl_upgrade!(
        CodeLineWidth,
        u8,
        [
            (10, Resources::from_bronze(10.)),
            (15, Resources::from_bronze(20.)),
            (20, Resources::default()),
        ]
    );

    impl_upgrade!(
        CodeLineCount,
        u8,
        [
            (3, Resources::from_bronze(10.)),
            (5, Resources::from_bronze(20.)),
            (20, Resources::default()),
        ]
    );

    impl_upgrade!(
        LoopStatements,
        bool,
        [
            (false, Resources::from_bronze(10.)),
            (true, Resources::default()),
        ]
    );

    #[derive(Debug, Default, Serialize, Deserialize, Clone)]
    pub struct Upgrades {
        compile_time: CompileTime,
        run_time: RunTime,
        speed_up_per_instruction: SpeedUpPerInstruction,
        bronze_per_instruction: BronzePerInstruction,
        code_line_width: CodeLineWidth,
        code_line_count: CodeLineCount,
        loop_statements: LoopStatements,
    }

    impl UpgradeCollection for Upgrades {
        fn upgrades(&self) -> impl Iterator<Item = &dyn Upgrade> {
            let x: [&dyn Upgrade; _] = [
                &self.compile_time,
                &self.run_time,
                &self.speed_up_per_instruction,
                &self.bronze_per_instruction,
                &self.code_line_width,
                &self.code_line_count,
                &self.loop_statements,
            ];
            x.into_iter()
        }

        fn upgrades_mut(&mut self) -> impl Iterator<Item = &mut dyn Upgrade> {
            let x: [&mut dyn Upgrade; _] = [
                &mut self.compile_time,
                &mut self.run_time,
                &mut self.speed_up_per_instruction,
                &mut self.bronze_per_instruction,
                &mut self.code_line_width,
                &mut self.code_line_count,
                &mut self.loop_statements,
            ];
            x.into_iter()
        }
    }
}
