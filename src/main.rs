use rust_htslib::{bam, bam::{Read,Record},tpool::ThreadPool};
use std::thread::available_parallelism;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = std::env::args().collect();

    let align_path = &args[1].to_string();
    let out_path = &args[2].to_string();
    let start_ref = &args[3].to_string();

    let mut reader = bam::IndexedReader::from_path(align_path)?;
    let header = bam::Header::from_template(reader.header());

    let refs: Vec<_> = header.to_hashmap()["SQ"].clone().into_iter().map(|m| m["SN"].clone()).collect();

    let start_ref_idx = refs.iter().position(|refname| refname == start_ref).unwrap();

    let end_ref_idx = if args.len() > 4 {
        let end_ref = &args[4].to_string();
        1 + refs.iter().position(|refname| refname == end_ref).unwrap()
    }
    else {
        refs.len() - 1
    };

    let mut writer = bam::Writer::from_path(out_path, &header, bam::Format::Bam)?;

    let mut record = Record::new();

    let num_cores = available_parallelism()?.get().try_into()?;
    let thread_pool = ThreadPool::new(num_cores)?;

    reader.set_thread_pool(&thread_pool)?;
    writer.set_thread_pool(&thread_pool)?;

    for rname in &refs[start_ref_idx..end_ref_idx] {
        println!("Copying records for {}", rname);

        reader.fetch(rname)?;

        while let Some(result) = reader.read(&mut record) {
            result?;
            writer.write(&record)?;
        }

        //for result in reader.records() {
        //    let record = result?;
        //    writer.write(&record)?;
        //}
    }

    Ok(())
}
