use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;
use std::collections::BTreeMap;
use std::io;

use clap::Args;
use ratatui::backend::CrosstermBackend;

use crate::events::*;
use crate::gitlab_ref::*;

type Term = Terminal<CrosstermBackend<io::Stdout>>;

#[derive(Debug, Args)]
pub struct PipelinesArgs {
    #[clap(trailing_var_arg = true)]
    gitlab_refs: Vec<GitlabRef>,
}

type Projects = BTreeMap<String, BTreeMap<String, Vec<bool>>>;

pub async fn run(gapi: gitlab::AsyncGitlab, mut terminal: Term, args: &PipelinesArgs) {
    let amount = terminal.size().unwrap().width / 3;

    let mut receivers: Vec<tokio::sync::mpsc::Receiver<crate::fetchers::BranchPipelineUpdate>> =
        args.gitlab_refs
            .iter()
            .map(|r| match &r {
                GitlabRef::Repo(repo) => todo!(),
                GitlabRef::Branch(repo, branch) => {
                    crate::fetchers::branch_pipelines(gapi.clone(), &repo, &branch, amount.into())
                }
            })
            .collect();

    let mut event_handler = EventHandler::new(250);
    let mut projects: Projects = BTreeMap::new();
    loop {
        for receiver in receivers.iter_mut() {
            // check if there is a new project update
            if let Ok(p) = receiver.try_recv() {
                let branches = projects.entry(p.project).or_insert(BTreeMap::new());
                let states = branches.entry(p.branch).or_insert(Vec::new());
                states.clear();
                states.extend_from_slice(&p.states);
            }
        }
        terminal
            .draw(|frame| {
                render(frame, &projects);
            })
            .expect("failed to draw frame");

        match event_handler.next().await {
            Event::Tick => {}
            Event::Quit => break,
            Event::Key(_) => break,
            Event::Resize(_, _) => terminal.autoresize().expect("failed to resize"),
        }
    }
}

fn render(frame: &mut Frame, projects: &Projects) {
    if projects.is_empty() {
        return;
    }

    let layouts = Layout::default()
        .constraints((0..projects.len()).into_iter().map(|_| Constraint::Min(3)))
        .flex(layout::Flex::Center)
        .split(frame.area());

    for ((project, branches), &layout) in projects.iter().zip(layouts.iter()) {
        render_project(frame, layout, project, branches);
    }
}
fn render_project(
    frame: &mut Frame,
    area: Rect,
    project: &str,
    branches: &BTreeMap<String, Vec<bool>>,
) {
    // render the project outline
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(project.to_string());
    frame.render_widget(&block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..branches.len()).into_iter().map(|_| Constraint::Max(3)))
        .flex(layout::Flex::Center)
        .split(block.inner(area));

    branches
        .iter()
        .zip(layout.iter())
        .for_each(|((branch, states), layout)| {
            let line: Line = states
                .iter()
                .map(|ok| match ok {
                    true => Span::styled("▄▄▄  ", Color::Green),
                    false => Span::styled("▄▄▄  ", Color::Red),
                })
                .collect::<Vec<Span>>()
                .into();
            let paragraph = Paragraph::new(line).block(Block::bordered().title(branch.clone()));

            frame.render_widget(paragraph, *layout);
        })
}
