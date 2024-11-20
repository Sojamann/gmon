use graphql_client::*;
use tokio::sync::mpsc::{channel, Receiver};

pub use crate::fetchers::pipelines::branch_pipelines_query::PipelineStatusEnum;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/pipelines.graphql",
    schema_path = "graphql/schema.json",
    variables_derives = "Debug",
    response_derives = "Deserialize,Serialize,PartialEq,Debug,Clone"
)]
struct BranchPipelinesQuery;

pub struct BranchPipelineUpdate {
    pub project: String,
    pub branch: Option<String>,
    pub states: Vec<PipelineStatusEnum>,
}

#[derive(Default, Debug, Clone)]
pub struct PipelinesQueryArgs {
    project: String,
    git_ref: Option<String>,
    pipeline_count: Option<i64>,
    pipeline_status: Option<PipelineStatusEnum>,
}

impl PipelinesQueryArgs {
    pub fn new(project: String) -> Self {
        return Self {
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

pub(crate) fn branch_pipelines(
    gapi: gitlab::AsyncGitlab,
    params: PipelinesQueryArgs,
) -> Receiver<BranchPipelineUpdate> {
    let (sender, receiver) = channel(1);

    let variables = <BranchPipelinesQuery as GraphQLQuery>::Variables {
        project: params.project.clone(),
        branch: params.git_ref.clone(),
        amount: params.pipeline_count,
    };
    let query = BranchPipelinesQuery::build_query(variables);
    tokio::spawn(async move {
        loop {
            let resp: <BranchPipelinesQuery as GraphQLQuery>::ResponseData = gapi
                .graphql::<BranchPipelinesQuery>(&query)
                .await
                .expect("some data");

            let states: Vec<_> = resp.project.into_iter()
                .flat_map(|p| p.pipelines)
                .flat_map(|p| p.nodes)
                .flatten()
                .filter_map(|p| p)
                .map(|p| p.status)
                .collect();

            sender
                .send(BranchPipelineUpdate {
                    project: params.project.clone(),
                    branch: params.git_ref.clone(),
                    states,
                })
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    return receiver;
}
