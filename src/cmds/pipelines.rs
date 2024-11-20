use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Paragraph, Padding};
use ratatui::Frame;
use std::io;

use clap::Args;

use crate::events::*;
use crate::fetchers::pipelines::BranchPipelineUpdate;
use crate::fetchers::pipelines::PipelineStatusEnum;
use crate::gitlab_ref::*;

#[derive(Debug, Args)]
pub struct PipelinesArgs {
    gitlab_ref: GitlabRef,
}

struct App {
    receiver: tokio::sync::mpsc::Receiver<BranchPipelineUpdate>,
    project: Option<BranchPipelineUpdate>,
}

impl App {
    fn new(gapi: gitlab::AsyncGitlab, args: &PipelinesArgs) -> Self {
        let receiver = match &args.gitlab_ref {
            GitlabRef::Repo(repo) => todo!(),
            GitlabRef::Branch(repo, branch) => {
                crate::fetchers::branch_pipelines(gapi.clone(), &repo, &branch, 40)
            }
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

pub async fn run(gapi: gitlab::AsyncGitlab, args: &PipelinesArgs) {
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    ratatui::crossterm::terminal::enable_raw_mode().expect("enable raw mode");
    let viewport = ratatui::Viewport::Inline(5);
    let mut terminal =
        ratatui::Terminal::with_options(backend, ratatui::TerminalOptions { viewport })
            .expect("terminal setup to work");

    let mut app = App::new(gapi, args);
    let mut event_handler = EventHandler::new(250);

    loop {
        app.update();
        // ignoring all errors
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

fn render(frame: &mut Frame, project: &BranchPipelineUpdate) {
    let project_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(project.project.clone());
    frame.render_widget(&project_block, frame.area());

    let project_content_area = project_block.inner(frame.area());

    let line: Line = project
        .states
        .iter()
        .map(|ok| match ok {
            PipelineStatusEnum::SUCCESS => Span::styled(" ███ ", Color::Green),
            PipelineStatusEnum::FAILED => Span::styled(" ███  ", Color::Red),
            PipelineStatusEnum::CREATED => Span::styled(" ███ ", Color::Gray),
            PipelineStatusEnum::SKIPPED => Span::styled("  »  ", Modifier::DIM),
            _ => Span::styled(" ███ ", Modifier::DIM),
        })
        .collect::<Vec<Span>>()
        .into();

    let paragraph = Paragraph::new(line).centered().block(
        Block::bordered()
            .padding(Padding::horizontal(3))
            .title(project.branch.clone()),
    );

    frame.render_widget(paragraph, project_content_area);
}
