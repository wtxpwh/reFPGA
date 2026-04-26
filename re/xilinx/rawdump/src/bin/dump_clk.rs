use clap::Parser;
use prjcombine_re_xilinx_rawdump::{Part, TkWire};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "dump_clk", about = "Dump Versal clock structure from rawdump.")]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let rd = Part::from_file(args.file)?;
    println!(
        "PART {} {} {:?} {}×{}",
        rd.part, rd.family, rd.source, rd.width, rd.height
    );
    let mut n2w: HashMap<_, Vec<_>> = HashMap::new();
    for (&crd, tile) in &rd.tiles {
        if !tile.name.starts_with("CLK") {
            continue;
        }
        let tk = &rd.tile_kinds[tile.kind];
        for &wni in tk.wires.keys() {
            let wn = &rd.wires[wni];
            if !wn.contains("DISTR") && !wn.contains("ROUTE") {
                continue;
            }
            if tk.wires.get(&wni).is_none() {
                println!("OOPS {tn} {wn}", tn = tile.name);
            }
            let w = tk.wires.get(&wni).unwrap().1;
            if let TkWire::Connected(cwi) = *w
                && let Some(ni) = tile.conn_wires.get(cwi)
            {
                n2w.entry(ni).or_default().push((crd, wn));
            }
        }
    }
    let mut conns = Vec::from_iter(n2w.into_values());
    for list in &mut conns {
        list.sort();
    }
    conns.sort();
    for list in conns {
        for (ci, wi) in list {
            println!("{ti:17} {wi:6}", ti = rd.tiles[&ci].name);
        }
        println!("---");
    }
    Ok(())
}
