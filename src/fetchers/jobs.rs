use graphql_client::*;
use tokio::sync::mpsc::{channel, Receiver};

pub use crate::fetchers::jobs::jobs_query::{CiJobStatus, PipelineStatusEnum};

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/jobs.graphql",
    schema_path = "graphql/schema.json",
    variables_derives = "Debug",
    response_derives = "Deserialize,Serialize,PartialEq,Debug,Clone"
)]
struct JobsQuery;

pub struct Project {
    pub id: String,
    pub full_path: String,
    pub pipelines: Vec<Pipeline>,
}

pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub git_ref: String,
    pub status: PipelineStatusEnum,
    pub stages: Vec<Stage>,
}

pub struct Stage {
    pub name: String,
    pub jobs: Vec<Job>,
}

pub struct Job {
    pub name: String,
    pub status: CiJobStatus,
}

#[derive(Default, Debug, Clone)]
pub struct JobQueryParams {
    project: String,
    git_ref: Option<String>,
    pipeline_count: Option<i64>,
    pipeline_status: Option<PipelineStatusEnum>,
}

impl JobQueryParams {
    pub fn new(project: String) -> Self {
        return JobQueryParams {
            project,
            ..Self::default()
        };
    }
    pub fn with_reference(mut self, reference: String) -> Self {
        self.git_ref = Some(reference);
        self
    }
    pub fn with_count(mut self, count: i64) -> Self {
        self.pipeline_count = Some(count);
        self
    }
    pub fn with_status(mut self, status: PipelineStatusEnum) -> Self {
        self.pipeline_status = Some(status);
        self
    }
}

pub(crate) fn jobs_pipelines(
    gapi: gitlab::AsyncGitlab,
    params: JobQueryParams,
) -> Receiver<Project> {
    let (sender, receiver) = channel(1);

    let variables = <JobsQuery as GraphQLQuery>::Variables {
        project: params.project.clone(),
        git_ref: params.git_ref.clone(),
        amount: params.pipeline_count.clone(),
    };
    let query = JobsQuery::build_query(variables);

    tokio::spawn(async move {
        loop {
            let resp: <JobsQuery as GraphQLQuery>::ResponseData =
                gapi.graphql::<JobsQuery>(&query).await.expect("some data");

            if resp.project.is_none() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let r_project = resp.project.unwrap();
            let pipelines: Vec<_> = r_project
                .pipelines
                .into_iter()
                .filter_map(|p| p.nodes)
                .flat_map(|p| p)
                .filter_map(|p| p)
                .map(|p| Pipeline {
                    id: p.id,
                    name: p.name.unwrap_or("no name".to_string()),
                    git_ref: p.ref_.unwrap_or("no name".to_string()),
                    status: p.status,
                    stages: p
                        .stages
                        .into_iter()
                        .filter_map(|s| s.nodes)
                        .flat_map(|s| s)
                        .filter_map(|s| s)
                        .map(|s| Stage {
                            name: s.name.unwrap_or("no name".to_string()),
                            jobs: s
                                .jobs
                                .into_iter()
                                .filter_map(|s| s.nodes)
                                .flat_map(|s| s)
                                .filter_map(|s| s)
                                .map(|j| Job {
                                    name: j.name.unwrap_or("no_name".to_string()),
                                    status: j.status.unwrap_or(CiJobStatus::CREATED),
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect();

            sender
                .send(Project {
                    id: r_project.id,
                    full_path: r_project.full_path,
                    pipelines,
                })
                .await
                .expect("sending");

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    return receiver;
}
