use druid::widget::*;
use druid::*;
use log::*;
use std::sync::Arc;

use crate::*;

const ADD_NEW_REPO: Selector = Selector::new("xyz.add-new-repo");
const FETCH_REPO_RELEASES: Selector = Selector::new("xyz.fetch-repo-releases");

pub(crate) fn build_gui() -> impl Widget<State> {
    Split::rows(
        Flex::column()
            .with_child(new_repo())
            .with_child(repo_list())
            .padding(10.0),
        repo_details(),
    )
    .split_point(0.3)
    .bar_size(5.0)
    .min_bar_area(11.0)
    .draggable(true)
    .controller(EventHndl::default())
}

#[derive(Debug, Default)]
struct EventHndl;
impl<W: Widget<State>> Controller<State, W> for EventHndl {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut State,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.selector == ADD_NEW_REPO => {
                let s = &cmd.get_object::<String>().expect("repo id not provided");
                match RepoId::parse(s) {
                    Ok(repo_id) => {
                        smol::run(async {
                            let repo = Repo::fetch_repo(&data.token, &repo_id).await.unwrap();
                            debug!("repo fetched: {:#?}", repo);
                            Arc::make_mut(&mut data.repos).push(repo);
                        });
                        ctx.request_layout();
                        ctx.request_paint();
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
            Event::Command(cmd) if cmd.selector == FETCH_REPO_RELEASES => {
                let repo = &cmd.get_object::<Repo>().expect("repo not provided");
                data.selected_repo = Some((*repo).clone());
                smol::run(async {
                    let mut releases = repo.fetch_releases(&data.token).await.unwrap();
                    releases.reverse();

                    *Arc::make_mut(&mut data.releases) = releases;

                    ctx.request_layout();
                    ctx.request_paint();
                });
            }
            _ => child.event(ctx, event, data, env),
        };
    }
}

fn new_repo() -> impl Widget<State> {
    TextBox::new()
        .with_placeholder("owner/repo")
        .controller(FocusOnLaunchCtrl::new())
        .controller(
            KeyCodeFilterCtrl::new(|code| {
                use druid::KeyCode::*;
                let valid = vec![
                    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, KeyQ, KeyW, KeyE,
                    KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP, KeyA, KeyS, KeyD, KeyF, KeyG, KeyH,
                    KeyJ, KeyK, KeyL, KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM, Slash, Minus,
                ];
                valid.contains(&code)
            })
            .with_meta_keys(),
        )
        .controller(ActionCtrl::new(|data: &mut String, ctx| {
            ctx.submit_command(Command::new(ADD_NEW_REPO, data.clone()), None);
        }))
        .expand_width()
        .padding(10.0)
        // FIXME
        .lens(State::dummy)
}

fn repo_list() -> impl Widget<State> {
    fn mk_col<G, S>(name: &str, get: G) -> Flex<Arc<Repos>>
    where
        G: FnOnce(&Repo) -> S + Copy + 'static,
        S: ToString,
    {
        Flex::column()
            .with_child(Label::new(name.to_string()))
            .with_child(List::new(move || {
                Label::dynamic(move |s: &Repo, _| get(s).to_string())
                    .padding(Insets::uniform_xy(40., 8.))
                    .on_click(|ctx, data, _| {
                        ctx.submit_command(Command::new(FETCH_REPO_RELEASES, data.clone()), None);
                    })
            }))
    }

    Scroll::new(
        Flex::row()
            .with_child(mk_col("Name", |s| s.repo_id.to_string()))
            .with_child(mk_col("Watchers", |s| s.watchers))
            .with_child(mk_col("Stars", |s| s.stargazers))
            .with_child(mk_col("Forks", |s| s.forks))
            .with_child(mk_col("Issues", |s| s.open_issues))
            .with_child(mk_col("Pull requests", |s| s.open_pull_requests)),
    )
    .vertical()
    .lens(State::repos)
}

fn repo_details() -> impl Widget<State> {
    fn mk_col<G, S>(name: &str, get: G) -> Flex<Arc<Releases>>
    where
        G: FnOnce(&Release) -> S + Copy + 'static,
        S: ToString,
    {
        Flex::column()
            .with_child(Label::new(name.to_string()))
            .with_child(List::new(move || {
                Label::dynamic(move |s: &Release, _| get(s).to_string())
                    .padding(Insets::uniform_xy(60., 8.))
            }))
    }

    let description = Label::dynamic(move |s: &State, _| {
        if let Some(repo) = &s.selected_repo {
            format!("Releases for repo '{}'", repo.repo_id)
        } else {
            "No releases".to_string()
        }
    })
    .padding(15.0);

    Flex::column().with_child(description).with_child(
        Scroll::new(
            Flex::row()
                .with_child(mk_col("Name", |s| {
                    s.name.as_ref().unwrap_or(&"".to_string()).clone()
                }))
                .with_child(mk_col("Tag name", |s| s.tag_name.clone()))
                .with_child(mk_col("Downloads", |s| {
                    s.assets
                        .iter()
                        .map(|a| a.downloads)
                        .sum::<i64>()
                        .to_string()
                })),
        )
        .vertical()
        .lens(State::releases),
    )
}
