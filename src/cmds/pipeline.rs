use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType};
use ratatui::Frame;
use std::io;

use clap::Args;

use crate::events::*;
use crate::fetchers::CiJobStatus;
use crate::gitlab_ref::*;
use crate::theme;
use crate::theme::Theme;

#[derive(Debug, Args)]
pub struct PipelineArgs {
    gitlab_ref: GitlabRef,
}

use crate::fetchers::{JobQueryParams, Project};

struct App {
    receiver: tokio::sync::mpsc::Receiver<Project>,
    /// there is only one project with one pipeline in here
    project: Option<Project>,
}

impl App {
    fn new(gapi: gitlab::AsyncGitlab, args: &PipelineArgs) -> Self {
        let receiver = match &args.gitlab_ref {
            GitlabRef::Repo(repo) => crate::fetchers::jobs_pipelines(
                gapi.clone(),
                JobQueryParams::new(repo.clone()).with_count(1),
            ),
            GitlabRef::Branch(repo, branch) => crate::fetchers::jobs_pipelines(
                gapi.clone(),
                JobQueryParams::new(repo.clone())
                    .with_reference(branch.clone())
                    .with_count(1),
            ),
        };

        App {
            receiver,
            project: None,
        }
    }

    fn update(&mut self) {
        // check if there is a new project update
        if let Ok(p) = self.receiver.try_recv() {
            self.project = Some(p);
        }
    }

    fn render(&self, frame: &mut Frame) {
        if let Some(p) = &self.project {
            render(frame, p);
        }
    }
}

pub async fn run(gapi: gitlab::AsyncGitlab, args: &PipelineArgs) {
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    ratatui::crossterm::terminal::enable_raw_mode().expect("enable raw mode");
    let viewport = ratatui::Viewport::Inline(15);
    let mut terminal =
        ratatui::Terminal::with_options(backend, ratatui::TerminalOptions { viewport })
            .expect("terminal setup to work");

    let mut app = App::new(gapi, args);
    let mut event_handler = EventHandler::new(250);

    loop {
        app.update();
        terminal
            .draw(|frame| app.render(frame))
            .expect("failed to draw frame");

        match event_handler.next().await {
            Event::Tick => {}
            Event::Quit => break,
            Event::Key(_) => break,
            Event::Resize(_, _) => terminal.autoresize().expect("failed to resize"),
        }
    }
}

fn render(frame: &mut Frame, project: &Project) {
    assert_eq!(project.pipelines.len(), 1);

    let project_block = theme::Catpuccin.block().title(project.full_path.clone());
    frame.render_widget(&project_block, frame.area());

    let project_content_area = project_block.inner(frame.area());

    let pipeline = &project.pipelines[0];
    let branch_block = theme::Catpuccin
        .block()
        .title(Line::from(pipeline.git_ref.clone()).left_aligned())
        .title(Line::from(pipeline.name.clone()).right_aligned());
    frame.render_widget(&branch_block, project_content_area);

    let stage_layouts = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            pipeline
                .stages
                .iter()
                .map(|s| Constraint::Min(u16::try_from(s.jobs.len()).expect("FUCK TOO MANY") + 2)),
        )
        .split(branch_block.inner(project_content_area));

    for (stage, stage_layout) in pipeline.stages.iter().zip(stage_layouts.iter()) {
        let [stage_name_layout, fill, line_layout] = Layout::horizontal([
            Constraint::Max(20),
            Constraint::Length(5),
            Constraint::Fill(1),
        ])
        .areas(*stage_layout);

        frame.render_widget(
            Line::from(stage.name.clone()).style(theme::Catpuccin::text()),
            stage_name_layout,
        );
        frame.render_widget(Block::new(), fill);

        let line = Line::from_iter(stage.jobs.iter().map(|j| match j.status {
            CiJobStatus::FAILED => Span::styled("⬤  ", theme::Catpuccin::red()),
            CiJobStatus::SUCCESS => Span::styled("⬤  ", theme::Catpuccin::green()),
            CiJobStatus::CREATED => Span::styled("⬤  ", theme::Catpuccin::blue()),
            CiJobStatus::SKIPPED => Span::styled("  »  ", theme::Catpuccin::text()),
            _ => Span::styled("⬤  ", theme::Catpuccin::text()),
        }));
        frame.render_widget(line, line_layout);
    }
}
