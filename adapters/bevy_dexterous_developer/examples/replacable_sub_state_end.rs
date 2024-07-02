use std::num::NonZero;

use bevy::{
    app::{AppExit, Startup, Update},
    prelude::*,
    MinimalPlugins,
};
use bevy_dexterous_developer::*;

fn terminal_runner(mut app: App) -> AppExit {
    app.update();
    eprintln!("Ready for Input");
    for line in std::io::stdin().lines() {
        let typed: String = line.unwrap_or_default();
        if typed == "exit" {
            println!("Exiting");
            return AppExit::Success;
        }
        app.update();
    }
    AppExit::Error(NonZero::<u8>::new(1).unwrap())
}

#[derive(States, Debug, Default, Hash, PartialEq, Eq, Clone)]
enum MyState {
    #[default]
    InitialState,
    AnotherState,
    ThirdState
}

impl ReplacableType for MyState {
    fn get_type_name() -> &'static str {
        "MySerializableResource"
    }

    fn to_vec(&self) -> bevy_dexterous_developer::Result<Vec<u8>> {
        let value = match self {
            MyState::InitialState => [0],
            MyState::AnotherState => [1],
            MyState::ThirdState => [2],
        };
        Ok(value.to_vec())
    }

    fn from_slice(val: &[u8]) -> bevy_dexterous_developer::Result<Self> {
        let value = if let Some(val) = val.get(0) {
            if *val == 1 {
                MyState::AnotherState
            } else if *val == 2 {
                MyState::ThirdState
            } else {
                MyState::InitialState
            }
        } else {
            MyState::InitialState
        };
        Ok(value)
    }
}

#[derive(SubStates, Debug, Default, Hash, PartialEq, Eq, Clone)]
#[source(MyState = MyState::AnotherState)]
enum MySubState {
    #[default]
    InitialState,
    AnotherState,
    ThirdState
}

impl ReplacableType for MySubState {
    fn get_type_name() -> &'static str {
        "MySubState"
    }

    fn to_vec(&self) -> bevy_dexterous_developer::Result<Vec<u8>> {
        let value = match self {
            MySubState::InitialState => [0],
            MySubState::AnotherState => [1],
            MySubState::ThirdState => [2]
        };
        Ok(value.to_vec())
    }

    fn from_slice(val: &[u8]) -> bevy_dexterous_developer::Result<Self> {
        let value = if let Some(val) = val.get(0) {
            if *val == 1 {
                MySubState::AnotherState
            } else if *val == 2 {
                MySubState::ThirdState
            } else {
                MySubState::InitialState
            }
        } else {
            MySubState::InitialState
        };
        Ok(value)
    }
}

reloadable_main!( bevy_main(initial_plugins) {
    App::new()
        .add_plugins(initial_plugins.initialize::<MinimalPlugins>())
        .set_runner(terminal_runner)
        .setup_reloadable_elements::<reloadable>()
        .run();
});

fn set_next_state(mut next_state: ResMut<NextState<MyState>>) {
    println!("In Initial State");
    next_state.set(MyState::AnotherState);
}

fn set_next_sub_state(mut next_state: ResMut<NextState<MySubState>>) {
    println!("In Initial Sub State");
    next_state.set(MySubState::AnotherState);
}

fn in_another_sub_state(mut next_state: ResMut<NextState<MySubState>>) {
    println!("In Another Sub State");
    next_state.set(MySubState::ThirdState);
}

fn in_third_sub_state(mut next_state: ResMut<NextState<MyState>>) {
    println!("In Third Sub State");
    next_state.set(MyState::ThirdState);
}

fn in_third_state() {
    println!("In Third State");
}

fn startup() {
    println!("Press Enter to Progress, or type 'exit' to exit");
}

reloadable_scope!(reloadable(app) {
    app
        .add_systems(Startup, startup)
        .add_systems(Update, (
            set_next_state.run_if(in_state(MyState::InitialState)),
            set_next_sub_state.run_if(in_state(MySubState::InitialState)),
            in_another_sub_state.run_if(in_state(MySubState::AnotherState)),
            in_third_sub_state.run_if(in_state(MySubState::ThirdState)),
            in_third_state.run_if(in_state(MyState::ThirdState)),
            ).chain()
        )
        .init_state::<MyState>()
        .add_sub_state::<MySubState>();
});
