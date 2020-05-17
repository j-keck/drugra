use druid::*;
use std::{env, sync::Arc};

mod druid_ext;
mod gui;
mod repo;

use druid_ext::*;
use gui::build_gui;
use repo::*;

#[derive(Data, Debug, Default, Lens, Clone)]
pub struct State {
    token: String,
    repos: Arc<Repos>,
    selected_repo: Option<Repo>,
    releases: Arc<Releases>,
    dummy: String,
}

pub(crate) fn main() {
    match env::args().nth(1) {
        Some(token) => {
            let mut state = State::default();
            state.token = token.to_string();

            let main_window = WindowDesc::new(build_gui);
            AppLauncher::with_window(main_window)
                .launch(state)
                .expect("launch failed");
        }
        None => eprintln!("<GITHUB_TOKEN> missing - usage: drugra <GITHUB_TOKEN>"),
    };
}
