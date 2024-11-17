use graphql_client::*;
use tokio::sync::mpsc::{channel, Receiver};

use crate::fetchers::pipelines::branch_pipelines_query::PipelineStatusEnum;

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
    pub states: Vec<bool>,
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

            let mut result = Vec::new();

            for project in resp.project.iter() {
                if let Some(pipelines) = project.pipelines.as_ref().and_then(|p| p.nodes.as_ref()) {
                    for pipeline in pipelines.iter().flatten() {
                        match pipeline.status {
                            PipelineStatusEnum::FAILED => result.push(false),
                            PipelineStatusEnum::SUCCESS => result.push(true),
                            _ => {}
                        }
                    }
                }
            }

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
