pub const GITHUB_ENV: [&str; 50] = [
    "INPUT_WORKDIR",
    "INPUT_KREW_TEMPLATE_FILE",
    "HOME",
    "GITHUB_JOB",
    "GITHUB_REF",
    "GITHUB_SHA",
    "GITHUB_REPOSITORY",
    "GITHUB_REPOSITORY_OWNER",
    "GITHUB_REPOSITORY_OWNER_ID",
    "GITHUB_RUN_ID",
    "GITHUB_RUN_NUMBER",
    "GITHUB_RETENTION_DAYS",
    "GITHUB_RUN_ATTEMPT",
    "GITHUB_REPOSITORY_ID",
    "GITHUB_ACTOR_ID",
    "GITHUB_ACTOR",
    "GITHUB_TRIGGERING_ACTOR",
    "GITHUB_WORKFLOW",
    "GITHUB_HEAD_REF",
    "GITHUB_BASE_REF",
    "GITHUB_EVENT_NAME",
    "GITHUB_SERVER_URL",
    "GITHUB_API_URL",
    "GITHUB_GRAPHQL_URL",
    "GITHUB_REF_NAME",
    "GITHUB_REF_PROTECTED",
    "GITHUB_REF_TYPE",
    "GITHUB_WORKFLOW_REF",
    "GITHUB_WORKFLOW_SHA",
    "GITHUB_WORKSPACE",
    "GITHUB_ACTION",
    "GITHUB_EVENT_PATH",
    "GITHUB_ACTION_REPOSITORY",
    "GITHUB_ACTION_REF",
    "GITHUB_PATH",
    "GITHUB_ENV",
    "GITHUB_STEP_SUMMARY",
    "GITHUB_STATE",
    "GITHUB_OUTPUT",
    "RUNNER_OS",
    "RUNNER_ARCH",
    "RUNNER_NAME",
    "RUNNER_ENVIRONMENT",
    "RUNNER_TOOL_CACHE",
    "RUNNER_TEMP",
    "RUNNER_WORKSPACE",
    "ACTIONS_RUNTIME_URL",
    "ACTIONS_RUNTIME_TOKEN",
    "ACTIONS_CACHE_URL",
    "ACTIONS_RESULTS_URL",
];

pub fn static_env_vars() -> Vec<(&'static str, &'static str)> {
    vec![("GITHUB_ACTIONS", "true"), ("CI", "true")]
}

pub fn static_vol_mounts() -> Vec<(&'static str, &'static str)> {
    vec![
        ("/home/runner/work/_temp/_github_home", "/github/home"),
        (
            "/home/runner/work/_temp/_github_workflow",
            "/github/workflow",
        ),
        (
            "/home/runner/work/_temp/_runner_file_commands",
            "/github/file_commands",
        ),
        (
            "/home/runner/work/conditioner/conditioner",
            "/github/workspace",
        ),
    ]
}
