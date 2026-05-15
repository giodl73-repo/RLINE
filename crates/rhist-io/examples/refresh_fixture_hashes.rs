use rhist_io::{default_fixture_dir, refresh_package_hashes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for fixture in [
        "l0-rename",
        "l0-missing-unit",
        "l1-split-merge",
        "l1-bad-weights",
        "l2-three-cycle",
        "real-ri-tract-unchanged",
    ] {
        let hash = refresh_package_hashes(default_fixture_dir(fixture))?;
        println!("{fixture} {hash}");
    }
    Ok(())
}
