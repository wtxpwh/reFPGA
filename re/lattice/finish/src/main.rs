use std::path::PathBuf;

use clap::Parser;
use prjcombine_ecp::db::Database;
use prjcombine_types::db::DumpFlags;

#[derive(Debug, Parser)]
struct Args {
    db: PathBuf,
    txt: PathBuf,
    geom: PathBuf,
    tiledb: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let geom = prjcombine_re_lattice_naming::Database::from_file(&args.geom).unwrap();

    let db = Database {
        chips: geom.chips.into_map_values(|(chip, _)| chip),
        bonds: geom.bonds,
        devices: geom.devices,
        int: geom.int,
    };
    db.to_file(&args.db).unwrap();
    db.dump(
        &mut std::fs::File::create(&args.txt).unwrap(),
        DumpFlags::all(),
    )
    .unwrap();
}
