use std::fs::read_dir;

use anyhow::{Context, Result};
use xshell::{cmd, Shell};

const SPINE_EXE: &'static str = "/Applications/Spine.app/Contents/MacOS/Spine";
const TEXTURE_PACKER_EXE: &'static str =
    "/Applications/TexturePacker.app/Contents/MacOS/TexturePacker";

#[derive(Debug, Default)]
pub struct Textures {
    color_textures: Vec<String>,
    normal_textures: Vec<String>,
    roughness_textures: Vec<String>,
}

fn find_textures(dir: &str) -> Result<Textures> {
    let mut textures = Textures::default();
    for entry in read_dir(dir)? {
        let file = format!(
            "{dir}/{}",
            entry?
                .file_name()
                .to_str()
                .context("Failed to convert OS string.")?
        );
        if file.ends_with(".png") {
            textures.color_textures.push(file);
        }
    }
    for entry in read_dir(format!("{}/maps", dir))? {
        let file = format!(
            "{dir}/maps/{}",
            entry?
                .file_name()
                .to_str()
                .context("Failed to convert OS string.")?
        );
        if file.ends_with("_n.png") {
            textures.normal_textures.push(file);
        } else if file.ends_with("_r.png") {
            textures.roughness_textures.push(file);
        }
    }
    Ok(textures)
}

pub fn export() -> Result<()> {
    let sh = Shell::new()?;

    let output_dir = "assets/pbr_showcase";

    let spine_file = "assets_src/pbr_showcase/pbr_showcase.spine";
    let export_json = "assets_src/pbr_showcase/export.json";
    cmd!(
        sh,
        "{SPINE_EXE} --input {spine_file} --output {output_dir} --export {export_json}"
    )
    .run()?;

    let textures_dir = "assets_src/pbr_showcase/textures";
    let Textures {
        color_textures,
        normal_textures,
        roughness_textures,
    } = find_textures(textures_dir)?;
    {
        let color_textures = color_textures.clone();
        cmd!(sh, "{TEXTURE_PACKER_EXE} {color_textures...} {normal_textures...} --scale 0.5 --sheet {output_dir}/pbr_showcase.png --data {output_dir}/pbr_showcase.atlas --format spine --pack-normalmaps --normalmap-suffix _n --normalmap-sheet {output_dir}/pbr_showcase_n.png").run()?;
    }
    cmd!(sh, "{TEXTURE_PACKER_EXE} {color_textures...} {roughness_textures...} --scale 0.5 --sheet {output_dir}/pbr_showcase.png --data {output_dir}/pbr_showcase.atlas --format spine --pack-normalmaps --normalmap-suffix _r --normalmap-sheet {output_dir}/pbr_showcase_r.png").run()?;

    Ok(())
}
