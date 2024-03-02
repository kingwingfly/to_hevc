use anyhow::Result;
use std::env;

struct Job<'a> {
    in_path: &'a str,
    out_path: &'a str,
}

fn to_hevc<'a>(job: Job<'a>) -> Result<()> {
    let Job { in_path, out_path } = job;
    let status = std::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-hwaccel",
            "cuda",
            "-hwaccel_output_format",
            "cuda",
            "-i",
            in_path,
            "-c:v",
            "hevc_nvenc",
            "-preset",
            "slow",
            out_path,
        ])
        // .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    match status.success() {
        true => {
            println!("{} converted", in_path);
            match std::fs::metadata(in_path)?.len() > std::fs::metadata(out_path)?.len() {
                true => std::fs::remove_file(in_path)?,
                false => std::fs::rename(in_path, out_path)?,
            }
        }
        false => println!("{} failed", in_path),
    }
    Ok(())
}

fn walk_dir(path: &str) -> Result<Vec<String>> {
    let mut paths = vec![];
    for e in std::fs::read_dir(path)?.filter_map(|e| e.ok()) {
        let path = e.path();
        if path.is_file()
            && new_mime_guess::from_path(&path).first() == Some("video/mp4".parse().unwrap())
        {
            paths.push(path.to_string_lossy().to_string());
        }
    }
    Ok(paths)
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    for in_dir in &args[1..] {
        let end = in_dir.split('/').last().unwrap();
        let out_dir = in_dir.replace(end, &format!("{end}_out"));
        println!("{} -> {}", in_dir, out_dir);
        std::fs::create_dir_all(out_dir)?;
        let in_paths: Vec<String> = walk_dir(in_dir)?;
        let out_paths = in_paths
            .iter()
            .map(|p| p.replace(end, &format!("{end}_out")))
            .collect::<Vec<_>>();
        for (in_path, out_path) in in_paths.into_iter().zip(out_paths.into_iter()) {
            println!("{} -> {}", in_path, out_path);
            let job = Job {
                in_path: &in_path,
                out_path: &out_path,
            };
            to_hevc(job)?;
        }
    }
    Ok(())
}
