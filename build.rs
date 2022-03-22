fn main()
{
    cfg_rust_features::emit!([
        "step_trait",
        "unwrap_infallible",
        "never_type",
        "unstable_features",
    ])
    .unwrap();
}
