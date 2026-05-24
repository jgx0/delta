use clap::CommandFactory;

#[test]
fn cli_builds() {
    let cmd = delta::cli::Cli::command();
    assert_eq!(cmd.get_name(), "delta");
}
