#[global_allocator]
static ALLOC: soksak_contract_terminal::bench::CountingAlloc =
    soksak_contract_terminal::bench::CountingAlloc::new();

mod common;

#[test]
#[ignore]
fn bench() {
    let report = soksak_contract_terminal::bench::run::<common::Unit>("shitty");
    println!("{}", report.to_line());
    if let Ok(directory) = std::env::var("SOKSAK_BENCH_OUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create benchmark directory");
        std::fs::write(directory.join("shitty.bench"), report.to_line())
            .expect("write Shitty benchmark");
    }
    soksak_contract_terminal::bench::assert_within_budget(&report);
}
