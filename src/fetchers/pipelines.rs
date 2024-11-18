use graphql_client::*;
use tokio::sync::mpsc::{channel, Receiver};

pub use crate::fetchers::pipelines::branch_pipelines_query::PipelineStatusEnum;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/pipelines.graphql",
    schema_path = "graphql/schema.json",
    variables_derives = "Debug",
    response_derives = "Deserialize,Serialize,PartialEq,Debug"
)]
struct BranchPipelinesQuery;

pub struct BranchPipelineUpdate {
    pub project: String,
    pub branch: String,
    pub states: Vec<PipelineStatusEnum>,
}

pub(crate) fn branch_pipelines(
    gapi: gitlab::AsyncGitlab,
    project: &str,
    branch: &str,
    amount: i64,
) -> Receiver<BranchPipelineUpdate> {
    let (sender, receiver) = channel(1);

    let project_name = project.to_string();
    let branch_name = branch.to_string();

    tokio::spawn(async move {
        let variables = <BranchPipelinesQuery as GraphQLQuery>::Variables {
            project: project_name.clone(),
            branch: branch_name.clone(),
            amount,
        };
        let query = BranchPipelinesQuery::build_query(variables);
        loop {
            let resp: <BranchPipelinesQuery as GraphQLQuery>::ResponseData = gapi
                .graphql::<BranchPipelinesQuery>(&query)
                .await
                .expect("some data");

            let result = resp.project.into_iter()
                .flat_map(|p| p.pipelines)
                .flat_map(|p| p.nodes)
                .flatten()
                .filter_map(|p| p)
                .map(|p| p.status)
                .collect();

            sender
                .send(BranchPipelineUpdate {
                    project: project_name.clone(),
                    branch: branch_name.clone(),
                    states: result,
                })
                .await
                .unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    return receiver;
}
